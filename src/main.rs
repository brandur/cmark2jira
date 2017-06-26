//! HTML renderer that takes an iterator of events as input.

extern crate pulldown_cmark;

use std::io::{self,Read};

use pulldown_cmark::{Event, Options, Parser, Tag};

fn translate_markdown(text: &str, opts: Options) {
    let mut footnote_def_num = 0;
    let mut footnote_ref_num = 0;

    let mut in_image = false;
    let mut in_ordered_list = false;
    let mut in_unordered_list = false;

    let p = Parser::new_ext(text, opts);
    for event in p {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::BlockQuote => print!("{{quote}}\n"),
                    Tag::Code => print!("{{{{"),
                    Tag::CodeBlock(lang) => {
                        if lang.is_empty() {
                            print!("{{code}}\n");
                        } else {
                            print!("{{code:{}}}\n", lang);
                        }
                    },
                    Tag::Emphasis => print!("_"),
                    Tag::FootnoteDefinition(_name) => {
                        print!("[{}]", footnote_def_num);
                        footnote_def_num += 1;
                    },
                    Tag::Header(level) => print!("h{}. ", level),
                    Tag::Image(dest, _title) => {
                        print!("!{}!", dest);
                        in_image = true;
                    },
                    Tag::Item => {
                        if in_ordered_list {
                            print!("# ");
                        } else if in_unordered_list {
                            print!("* ");
                        }
                    },
                    Tag::Link(_dest, _title) => print!("["),
                    Tag::List(None) => {
                        in_unordered_list = true;
                    },
                    Tag::List(_count) => {
                        in_ordered_list = true;
                    },
                    Tag::Paragraph => (),
                    // Four dashes instead of three. Way to show your clever individuality Atlassian!
                    Tag::Rule => print!("----\n\n"),
                    Tag::Strong => print!("*"),

                    // Sorry, tables not handled at all right now.
                    Tag::Table(_align) => (),
                    Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                };
            }
            Event::End(tag) => {
                match tag {
                    Tag::BlockQuote => print!("{{quote}}\n\n"),
                    Tag::Code => print!("}}}}"),
                    Tag::CodeBlock(_lang) => print!("{{code}}\n\n"),
                    Tag::Emphasis => print!("_"),
                    Tag::FootnoteDefinition(_name) => (),
                    Tag::Header(_level) => print!("\n\n"),
                    Tag::Image(_dest, _title) => {
                        in_image = false;
                    },
                    Tag::Item => print!("\n"),
                    Tag::Link(dest, _title) => print!("|{}]", dest),
                    Tag::List(None) => {
                        in_unordered_list = false;
                        print!("\n");
                    },
                    Tag::List(_count) => {
                        in_ordered_list = false;
                        print!("\n");
                    },
                    Tag::Rule => (),
                    Tag::Paragraph => print!("\n\n"),
                    Tag::Strong => print!("*"),
                    Tag::Table(_align) => (),
                    Tag::TableHead | Tag::TableRow | Tag::TableCell => (),
                };
            },
            Event::FootnoteReference(_name) => {
                print!("[{}]", footnote_ref_num);
                footnote_ref_num += 1;
            },
            Event::Html(content) |
            Event::InlineHtml(content) |
            Event::Text(content) => {
                // Image titles come out rendered as text rather than as an attribute for an image tag, so we need to special case them so as not to print.
                if !in_image {
                    print!("{}", content);
                }
            },
            Event::HardBreak => print!("\n\n"),
            Event::SoftBreak => print!("\n"),

        }
    }
}

fn main() {
    let opts = Options::empty();
    let mut input = String::new();
    if let Err(why) = io::stdin().read_to_string(&mut input) {
        panic!("couldn't read from stdin: {}", why)
    }
    translate_markdown(&input, opts);
}
