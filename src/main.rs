//! A very simple program that reads CommonMark (Markdown) content from stdin,
//! translates it to JIRA's special markup language, and prints the result on
//! stdout. This is useful for enabling Vim-based workflows that allows users
//! of Atlassian software to work purely with Markdown.

extern crate pulldown_cmark;

use std::io::{self,Read};

use pulldown_cmark::{Event, Options, Parser, Tag};

/// Renderer that converts input CommonMark to output Jira markup.
struct JIRARenderer<'a> {
    pub buf: &'a mut String,
    pub input: &'a str,

    in_image: bool,
    in_ordered_list: bool,
    in_unordered_list: bool,
    num_queued_newlines: i64,
}

impl<'a> JIRARenderer<'a> {
    /// Runs the renderer and converts input CommonMark to output Jira markup.
    /// The result is left in buf.
    pub fn run(&mut self) {
        let opts = Options::empty();
        let p = Parser::new_ext(self.input, opts);
        for event in p {
            self.process_event(event);
        }
    }

    // Appends content to buf after first adding any newlines that are queued
    // to be appended beforehand.
    fn append(&mut self, s: &str) {
        match self.num_queued_newlines {
            0 => (),
            1 => self.append_newline_if_not_present(),
            2 => {
                self.append_newline_if_not_present();
                self.buf.push('\n');
            },
            _ => panic!("No more than two newlines should ever be queued"),
        }
        self.num_queued_newlines = 0;
        self.buf.push_str(s);
    }

    // Appends a newline to buf, but only if it doesn't already end with a
    // newline.
    fn append_newline_if_not_present(&mut self) {
        if !(self.buf.is_empty() || self.buf.ends_with('\n')) {
            self.buf.push('\n');
        }
    }

    // Queues up a single newline to be written into the buffer. A newline is
    // only appended in the case more content is added to the buffer so that
    // subsequent calls can control that spacing and so that we're not left
    // with trailing whitespace when we finish rendering.
    fn ensure_double_space(&mut self) {
        self.num_queued_newlines = 2;
    }

    // Queues up a double newline to be written into the buffer. Newlines are
    // only appended in the case more content is added to the buffer so that
    // subsequent calls can control that spacing and so that we're not left
    // with trailing whitespace when we finish rendering.
    fn ensure_single_space(&mut self) {
        self.num_queued_newlines = 1;
    }

    fn process_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.process_event_start(tag),
            Event::End(tag) => self.process_event_end(tag),
            Event::Html(content) |
            Event::InlineHtml(content) |
            Event::Text(content) => {
                // Image titles come out rendered as text rather than as an
                // attribute for an image tag, so we need to special case them
                // so as not to print.
                if !self.in_image {
                    self.append(&*format!("{}", content));
                }
            },
            Event::HardBreak => self.ensure_double_space(),
            Event::SoftBreak => self.ensure_single_space(),

            // Tables and footnotes need to be specially enabled in the
            // CommonMark parser. We have them set to just pass through as
            // text, so these events can all be safely ignored (see the tests
            // below).
            Event::FootnoteReference(_name) => (),
        }
    }

    fn process_event_start(&mut self, tag: Tag) {
        match tag {
            Tag::BlockQuote => {
                self.append("{quote}");
                self.ensure_single_space();
            },
            Tag::Code => self.append("{{"),
            Tag::CodeBlock(lang) => {
                if lang.is_empty() {
                    self.append("{code}");
                } else {
                    self.append(&*format!("{{code:{}}}", lang));
                }
                self.ensure_single_space();
            },
            Tag::Emphasis => self.append("_"),
            Tag::Header(level) => self.append(&*format!("h{}. ", level)),
            Tag::Image(dest, _title) => {
                self.append(&*format!("!{}!", dest));
                self.in_image = true;
            },
            Tag::Item => {
                if self.in_ordered_list {
                    self.append("# ");
                } else if self.in_unordered_list {
                    self.append("* ");
                }
            },
            Tag::Link(_dest, _title) => self.append("["),
            Tag::List(None) => {
                self.in_unordered_list = true;
            },
            Tag::List(_count) => {
                self.in_ordered_list = true;
            },
            Tag::Paragraph => (),
            // Four dashes instead of three. Way to show your clever individuality Atlassian!
            Tag::Rule => {
                self.append("----");
                self.num_queued_newlines = 2;
            },
            Tag::Strong => self.append("*"),

            // Tables and footnotes need to be specially enabled in the
            // CommonMark parser. We have them set to just pass through as
            // text, so these events can all be safely ignored (see the tests
            // below).
            Tag::FootnoteDefinition(_name) => (),
            Tag::Table(_align) => (),
            Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
        }
    }

    fn process_event_end(&mut self, tag: Tag) {
        match tag {
            Tag::BlockQuote => {
                self.ensure_single_space();
                self.append("{quote}");
                self.ensure_double_space();
            },
            Tag::Code => self.append("}}"),
            Tag::CodeBlock(_lang) => {
                self.append("{code}");
                self.ensure_double_space();
            },
            Tag::Emphasis => self.append("_"),
            Tag::Header(_level) => {
                self.ensure_double_space();
            },
            Tag::Image(_dest, _title) => {
                self.in_image = false;
            },
            Tag::Item => {
                self.ensure_single_space();
            },
            Tag::Link(dest, _title) => self.append(&*format!("|{}]", dest)),
            Tag::List(None) => {
                self.in_unordered_list = false;
                self.ensure_double_space();
            },
            Tag::List(_count) => {
                self.in_ordered_list = false;
                self.ensure_double_space();
            },
            Tag::Rule => (),
            Tag::Paragraph => self.ensure_double_space(),
            Tag::Strong => self.append("*"),

            // Tables and footnotes need to be specially enabled in the
            // CommonMark parser. We have them set to just pass through as
            // text, so these events can all be safely ignored (see the tests
            // below).
            Tag::FootnoteDefinition(_name) => (),
            Tag::Table(_align) => (),
            Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
        };
    }
}

/// Renders an input CommonMark string to output JIRA markup.
fn render(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    {
        let mut renderer = JIRARenderer {
            buf: &mut buf,
            in_image: false,
            in_ordered_list: false,
            in_unordered_list: false,
            input: &s,
            num_queued_newlines: 0,
        };
        renderer.run();
    }
    return buf;
}

fn main() {
    let mut input = String::new();
    if let Err(why) = io::stdin().read_to_string(&mut input) {
        panic!("couldn't read from stdin: {}", why)
    }
    print!("{}", render(input.as_str()));
}

#[test]
fn test_translate_basic() {
    let input = r##"# Title One"##;
    let expected = r##"h1. Title One"##;
    assert_eq!(expected, render(input));
}

#[test]
fn test_translate_complex() {
    let input = r##"# Title One

This is a sample paragraph that has some text which is *emphasized* and some
other text which is **strong**. This is ***emphasized and strong***.

This paragraph [has a link](https://example.com).

This paragraph has `some code`.

![An image](https://example.com)

---

## Subsection

This is a subsection.

### Sub-subsection

This is a section nested below the subsection above.

## Ordered Lists

1. Item one.
2. Item two.
3. Item three.

## Unordered Lists

* Item one.
* Item two.
* Item three.

## Quotes

This is a single paragraph quote:

> Paragraph 1.

And this is a multi-paragraph quote:

> Paragraph 1.
>
> Paragraph 2.

## Code

Here is a code block without language:

```
cat "*strong*" | cmark2jira
```

And here is one with a language:

``` ruby
def foo
  puts "bar"
end
```
"##;
    let expected = r##"h1. Title One

This is a sample paragraph that has some text which is _emphasized_ and some
other text which is *strong*. This is *_emphasized and strong_*.

This paragraph [has a link|https://example.com].

This paragraph has {{some code}}.

!https://example.com!

----

h2. Subsection

This is a subsection.

h3. Sub-subsection

This is a section nested below the subsection above.

h2. Ordered Lists

# Item one.
# Item two.
# Item three.

h2. Unordered Lists

* Item one.
* Item two.
* Item three.

h2. Quotes

This is a single paragraph quote:

{quote}
Paragraph 1.
{quote}

And this is a multi-paragraph quote:

{quote}
Paragraph 1.

Paragraph 2.
{quote}

h2. Code

Here is a code block without language:

{code}
cat "*strong*" | cmark2jira
{code}

And here is one with a language:

{code:ruby}
def foo
  puts "bar"
end
{code}"##;

    // note that these only print in the event of a failure
    let actual = render(input);
    println!("*** expected ***\n{}", expected);
    println!("*** actual ***\n{}", actual);
    assert_eq!(expected, actual);
}

#[test]
fn test_translate_ignores_footnotes() {
    let input = r##"This is a paragraph of content [1] with a footnote.

[1] This is the footnote definition."##;
    let expected = input;
    assert_eq!(expected, render(input));
}

#[test]
fn test_translate_ignores_tables() {
    let input = r##"This is some CommonMark content with a table.

| Header  | Another header |
|---------|----------------|
| field 1 | something      |
| field 2 | something else |"##;
    let expected = input;
    assert_eq!(expected, render(input));
}
