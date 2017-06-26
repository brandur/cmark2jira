//! HTML renderer that takes an iterator of events as input.

extern crate pulldown_cmark;

use std::io::{self,Read};

use pulldown_cmark::{Event, Options, Parser, Tag};

fn fresh_line(buf: &mut String) {
    if !(buf.is_empty() || buf.ends_with('\n')) {
        buf.push('\n');
    }
}

fn translate(text: &str, buf: &mut String) {
    let mut footnote_def_num = 0;
    let mut footnote_ref_num = 0;

    let mut in_image = false;
    let mut in_ordered_list = false;
    let mut in_unordered_list = false;

    let opts = Options::empty();
    let p = Parser::new_ext(text, opts);
    for event in p {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::BlockQuote => buf.push_str("{quote}\n"),
                    Tag::Code => buf.push_str("{{"),
                    Tag::CodeBlock(lang) => {
                        if lang.is_empty() {
                            buf.push_str("{code}\n");
                        } else {
                            buf.push_str(&*format!("{{code:{}}}\n", lang));
                        }
                    },
                    Tag::Emphasis => buf.push_str("_"),
                    Tag::FootnoteDefinition(_name) => {
                        buf.push_str(&*format!("[{}]", footnote_def_num));
                        footnote_def_num += 1;
                    },
                    Tag::Header(level) => buf.push_str(&*format!("h{}. ", level)),
                    Tag::Image(dest, _title) => {
                        buf.push_str(&*format!("!{}!", dest));
                        in_image = true;
                    },
                    Tag::Item => {
                        if in_ordered_list {
                            buf.push_str("# ");
                        } else if in_unordered_list {
                            buf.push_str("* ");
                        }
                    },
                    Tag::Link(_dest, _title) => buf.push_str("["),
                    Tag::List(None) => {
                        in_unordered_list = true;
                    },
                    Tag::List(_count) => {
                        in_ordered_list = true;
                    },
                    Tag::Paragraph => (),
                    // Four dashes instead of three. Way to show your clever individuality Atlassian!
                    Tag::Rule => buf.push_str("----\n\n"),
                    Tag::Strong => buf.push_str("*"),

                    // Sorry, tables not handled at all right now.
                    Tag::Table(_align) => (),
                    Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                };
            }
            Event::End(tag) => {
                match tag {
                    Tag::BlockQuote => {
                        fresh_line(buf);
                        buf.push_str("{quote}\n\n")
                    },
                    Tag::Code => buf.push_str("}}"),
                    Tag::CodeBlock(_lang) => {
                        fresh_line(buf);
                        buf.push_str("{code}\n\n")
                    },
                    Tag::Emphasis => buf.push_str("_"),
                    Tag::FootnoteDefinition(_name) => (),
                    Tag::Header(_level) => {
                        fresh_line(buf);
                        buf.push_str("\n")
                    },
                    Tag::Image(_dest, _title) => {
                        in_image = false;
                    },
                    Tag::Item => fresh_line(buf),
                    Tag::Link(dest, _title) => buf.push_str(&*format!("|{}]", dest)),
                    Tag::List(None) => {
                        in_unordered_list = false;
                        fresh_line(buf);
                        buf.push_str("\n")
                    },
                    Tag::List(_count) => {
                        in_ordered_list = false;
                        fresh_line(buf);
                        buf.push_str("\n")
                    },
                    Tag::Rule => (),
                    Tag::Paragraph => {
                        fresh_line(buf);
                        buf.push_str("\n")
                    },
                    Tag::Strong => buf.push_str("*"),
                    Tag::Table(_align) => (),
                    Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                };
            },
            Event::FootnoteReference(_name) => {
                buf.push_str(&*format!("[{}]", footnote_ref_num));
                footnote_ref_num += 1;
            },
            Event::Html(content) |
            Event::InlineHtml(content) |
            Event::Text(content) => {
                // Image titles come out rendered as text rather than as an
                // attribute for an image tag, so we need to special case them
                // so as not to print.
                if !in_image {
                    buf.push_str(&*format!("{}", content));
                }
            },
            Event::HardBreak => buf.push_str("\n\n"),
            Event::SoftBreak => buf.push_str("\n"),

        }
    }
}

fn main() {
    let mut input = String::new();
    if let Err(why) = io::stdin().read_to_string(&mut input) {
        panic!("couldn't read from stdin: {}", why)
    }
    let mut buf = String::with_capacity(input.len());
    translate(&input, &mut buf);
    print!("{}", buf);
}

#[test]
fn test_translate_basic() {
    let input = r##"# Title One
"##;
    let expected = r##"h1. Title One

"##;
    let mut buf = String::with_capacity(input.len());
    translate(&input, &mut buf);
    assert_eq!(expected, buf);
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
{code}

"##;
    let mut buf = String::with_capacity(input.len());
    translate(&input, &mut buf);

    // note that these only print in the event of a failure
    println!("*** expected ***\n{}", expected);
    println!("*** actual ***\n{}", buf);

    assert_eq!(expected, buf);
}
