//! A very simple program that reads CommonMark (Markdown) content from stdin,
//! translates it to JIRA's special markup language, and prints the result on
//! stdout. This is useful for enabling Vim-based workflows that allows users
//! of Atlassian software to work purely with Markdown.

extern crate pulldown_cmark;

use std::io::{self,Read};

use pulldown_cmark::{Event, Options, Parser, Tag};

struct JIRARenderer<'a> {
    buf: &'a mut String,
    input: &'a str,
    num_queued_newlines: i64,
}

impl<'a> JIRARenderer<'a> {
    fn append(&mut self, s: &str) {
        if self.num_queued_newlines > 0 {
            self.fresh_line();
            self.num_queued_newlines -= 1;

            for _i in 0..self.num_queued_newlines {
                self.buf.push('\n');
                self.num_queued_newlines -= 1;
            }
        }
        self.buf.push_str(s);
    }

    fn fresh_line(&mut self) {
        if !(self.buf.is_empty() || self.buf.ends_with('\n')) {
            self.buf.push('\n');
        }
    }

    fn run(&mut self) {
        let mut footnote_def_num = 0;
        let mut footnote_ref_num = 0;

        let mut in_image = false;
        let mut in_ordered_list = false;
        let mut in_unordered_list = false;

        let opts = Options::empty();
        let p = Parser::new_ext(self.input, opts);
        for event in p {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::BlockQuote => {
                            self.append("{quote}");
                            self.num_queued_newlines = 1;
                        },
                        Tag::Code => self.append("{{"),
                        Tag::CodeBlock(lang) => {
                            if lang.is_empty() {
                                self.append("{code}");
                            } else {
                                self.append(&*format!("{{code:{}}}", lang));
                            }
                            self.num_queued_newlines = 1;
                        },
                        Tag::Emphasis => self.append("_"),
                        Tag::FootnoteDefinition(_name) => {
                            self.append(&*format!("[{}]", footnote_def_num));
                            footnote_def_num += 1;
                        },
                        Tag::Header(level) => self.append(&*format!("h{}. ", level)),
                        Tag::Image(dest, _title) => {
                            self.append(&*format!("!{}!", dest));
                            in_image = true;
                        },
                        Tag::Item => {
                            if in_ordered_list {
                                self.append("# ");
                            } else if in_unordered_list {
                                self.append("* ");
                            }
                        },
                        Tag::Link(_dest, _title) => self.append("["),
                        Tag::List(None) => {
                            in_unordered_list = true;
                        },
                        Tag::List(_count) => {
                            in_ordered_list = true;
                        },
                        Tag::Paragraph => (),
                        // Four dashes instead of three. Way to show your clever individuality Atlassian!
                        Tag::Rule => {
                            self.append("----");
                            self.num_queued_newlines = 2;
                        },
                        Tag::Strong => self.append("*"),

                        // Sorry, tables not handled at all right now.
                        Tag::Table(_align) => (),
                        Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                    };
                }
                Event::End(tag) => {
                    match tag {
                        Tag::BlockQuote => {
                            self.num_queued_newlines = 1;
                            self.append("{quote}");
                            self.num_queued_newlines = 2;
                        },
                        Tag::Code => self.append("}}"),
                        Tag::CodeBlock(_lang) => {
                            self.append("{code}");
                            self.num_queued_newlines = 2;
                        },
                        Tag::Emphasis => self.append("_"),
                        Tag::FootnoteDefinition(_name) => (),
                        Tag::Header(_level) => {
                            self.num_queued_newlines = 2;
                        },
                        Tag::Image(_dest, _title) => {
                            in_image = false;
                        },
                        Tag::Item => {
                            self.num_queued_newlines = 1;
                        },
                        Tag::Link(dest, _title) => self.append(&*format!("|{}]", dest)),
                        Tag::List(None) => {
                            in_unordered_list = false;
                            self.num_queued_newlines = 2;
                        },
                        Tag::List(_count) => {
                            in_ordered_list = false;
                            self.num_queued_newlines = 2;
                        },
                        Tag::Rule => (),
                        Tag::Paragraph => {
                            self.num_queued_newlines = 2;
                        },
                        Tag::Strong => self.append("*"),
                        Tag::Table(_align) => (),
                        Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                    };
                },
                Event::FootnoteReference(_name) => {
                    self.append(&*format!("[{}]", footnote_ref_num));
                    footnote_ref_num += 1;
                },
                Event::Html(content) |
                Event::InlineHtml(content) |
                Event::Text(content) => {
                    // Image titles come out rendered as text rather than as an
                    // attribute for an image tag, so we need to special case them
                    // so as not to print.
                    if !in_image {
                        self.append(&*format!("{}", content));
                    }
                },
                Event::HardBreak => {
                    self.num_queued_newlines = 2;
                },
                Event::SoftBreak => {
                    self.num_queued_newlines = 1;
                },
            }
        }
    }
}

fn render(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    {
        let mut renderer = JIRARenderer {
            buf: &mut buf,
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
