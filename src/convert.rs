use crate::{
    flex::{
        BoxLayout, Bubble, BubbleType, ButtonHeight, ButtonStyle, Component, FlexBox, FlexBoxType,
        FlexMessage, FlexMessageType, FontStyle, FontWeight, Span, SpanType, TextDecoration,
        UriAction, UriActionType,
    },
    parser::{Fragment, next_fragment},
};

/// Styling context accumulated during recursive parsing.
#[derive(Debug, Clone, Default)]
struct SpanStyle {
    weight: Option<FontWeight>,
    style: Option<FontStyle>,
    decoration: Option<TextDecoration>,
    color: Option<&'static str>,
    size: Option<&'static str>,
}

/// A collected link to be rendered as a footer button.
#[derive(Debug, Clone)]
struct CollectedLink {
    label: String,
    url: String,
}

/// Intermediate block produced during top-level parsing.
enum Block {
    /// Inline content to be rendered as a text component with spans.
    Text(Vec<Span>),
    /// Code block to be rendered as its own text component.
    CodeBlock(String),
}

/// Convert Telegram MarkdownV2 text into a Flex Message.
pub fn convert(text: &str) -> FlexMessage {
    let (blocks, links) = parse_blocks(text);
    let alt_text = strip_markdown(text);

    let mut body_contents: Vec<Component> = Vec::new();

    for block in blocks {
        match block {
            Block::Text(spans) => {
                if !spans.is_empty() {
                    body_contents.push(Component::Text {
                        wrap: true,
                        contents: spans,
                    });
                }
            }
            Block::CodeBlock(code) => {
                body_contents.push(Component::Text {
                    wrap: true,
                    contents: vec![Span {
                        type_: SpanType::Span,
                        text: code,
                        weight: None,
                        style: None,
                        decoration: None,
                        color: Some("#CC0000".to_owned()),
                        size: Some("sm".to_owned()),
                    }],
                });
            }
        }
    }

    // If body is empty, add an empty text component
    if body_contents.is_empty() {
        body_contents.push(Component::Text {
            wrap: true,
            contents: vec![Span::plain(String::new())],
        });
    }

    let footer = if links.is_empty() {
        None
    } else {
        let buttons = links
            .into_iter()
            .map(|link| Component::Button {
                action: UriAction {
                    type_: UriActionType::Uri,
                    label: link.label,
                    uri: link.url,
                },
                style: ButtonStyle::Link,
                height: ButtonHeight::Sm,
            })
            .collect();

        Some(FlexBox {
            type_: FlexBoxType::Box,
            layout: BoxLayout::Vertical,
            contents: buttons,
            spacing: Some("sm".to_owned()),
        })
    };

    FlexMessage {
        type_: FlexMessageType::Flex,
        alt_text,
        contents: Bubble {
            type_: BubbleType::Bubble,
            body: FlexBox {
                type_: FlexBoxType::Box,
                layout: BoxLayout::Vertical,
                contents: body_contents,
                spacing: Some("md".to_owned()),
            },
            footer,
        },
    }
}

/// Parse input into blocks (text runs and code blocks) + collected links.
fn parse_blocks(text: &str) -> (Vec<Block>, Vec<CollectedLink>) {
    let mut blocks: Vec<Block> = Vec::new();
    let mut current_spans: Vec<Span> = Vec::new();
    let mut links: Vec<CollectedLink> = Vec::new();

    let mut input = text;
    while !input.is_empty() {
        let fragment = next_fragment(&mut input);

        match fragment {
            Fragment::CodeBlock(content) => {
                // Flush current inline spans
                if !current_spans.is_empty() {
                    blocks.push(Block::Text(std::mem::take(&mut current_spans)));
                }

                // Extract code (skip language line, trim trailing newline)
                let code = if let Some(newline_pos) = content.find('\n') {
                    content[newline_pos + 1..].trim_end_matches('\n')
                } else {
                    content
                };
                blocks.push(Block::CodeBlock(code.to_owned()));
            }
            _ => {
                collect_fragment_spans(
                    fragment,
                    &SpanStyle::default(),
                    &mut current_spans,
                    &mut links,
                );
            }
        }
    }

    // Flush remaining spans
    if !current_spans.is_empty() {
        blocks.push(Block::Text(current_spans));
    }

    (blocks, links)
}

/// Process a single fragment into spans, with the given inherited style.
fn collect_fragment_spans(
    fragment: Fragment<'_>,
    parent_style: &SpanStyle,
    spans: &mut Vec<Span>,
    links: &mut Vec<CollectedLink>,
) {
    match fragment {
        Fragment::Plain(c) => {
            push_char(spans, parent_style, c);
        }
        Fragment::Escaped(c) => {
            push_char(spans, parent_style, c);
        }
        Fragment::InlineCode(content) => {
            spans.push(Span {
                type_: SpanType::Span,
                text: content.to_owned(),
                weight: None,
                style: None,
                decoration: None,
                color: Some("#CC0000".to_owned()),
                size: Some("sm".to_owned()),
            });
        }
        Fragment::Link { text, url } => {
            // Render link text inline with blue underline styling
            let link_style = SpanStyle {
                color: Some("#1689FC"),
                decoration: Some(TextDecoration::Underline),
                ..parent_style.clone()
            };
            collect_inline_spans(text, &link_style, spans, links);

            // Collect for footer button
            let label = strip_markdown(text);
            links.push(CollectedLink {
                label,
                url: url.to_owned(),
            });
        }
        Fragment::Formatted { delim, content } => {
            let child_style = apply_delim(parent_style, delim);
            collect_inline_spans(content, &child_style, spans, links);
        }
        Fragment::CodeBlock(_) => {
            // Should not happen at this level — handled in parse_blocks
        }
    }
}

/// Recursively parse inline content and collect spans.
fn collect_inline_spans(
    text: &str,
    style: &SpanStyle,
    spans: &mut Vec<Span>,
    links: &mut Vec<CollectedLink>,
) {
    let mut input = text;
    while !input.is_empty() {
        let fragment = next_fragment(&mut input);
        collect_fragment_spans(fragment, style, spans, links);
    }
}

/// Try to append a character to the last span if it has matching style,
/// otherwise start a new span.
fn push_char(spans: &mut Vec<Span>, style: &SpanStyle, c: char) {
    if let Some(last) = spans.last_mut()
        && span_matches_style(last, style)
    {
        last.text.push(c);
        return;
    }
    spans.push(make_span(style, c.to_string()));
}

fn span_matches_style(span: &Span, style: &SpanStyle) -> bool {
    span.weight == style.weight
        && span.style == style.style
        && span.decoration == style.decoration
        && span.color.as_deref() == style.color
        && span.size.as_deref() == style.size
}

fn make_span(style: &SpanStyle, text: String) -> Span {
    Span {
        type_: SpanType::Span,
        text,
        weight: style.weight,
        style: style.style,
        decoration: style.decoration,
        color: style.color.map(|s| s.to_owned()),
        size: style.size.map(|s| s.to_owned()),
    }
}

fn apply_delim(parent: &SpanStyle, delim: &str) -> SpanStyle {
    let mut s = parent.clone();
    match delim {
        "*" => s.weight = Some(FontWeight::Bold),
        "_" => s.style = Some(FontStyle::Italic),
        "__" => s.decoration = Some(TextDecoration::Underline),
        "~" => s.decoration = Some(TextDecoration::LineThrough),
        "||" => s.color = Some("#CCCCCC"),
        _ => {}
    }
    s
}

/// Strip all markdown formatting, returning plain text for altText.
fn strip_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut input = text;

    while !input.is_empty() {
        let fragment = next_fragment(&mut input);
        match fragment {
            Fragment::Plain(c) | Fragment::Escaped(c) => out.push(c),
            Fragment::CodeBlock(content) => {
                // Skip language line, trim trailing newline
                if let Some(newline_pos) = content.find('\n') {
                    out.push_str(content[newline_pos + 1..].trim_end_matches('\n'));
                } else {
                    out.push_str(content);
                }
            }
            Fragment::InlineCode(content) => out.push_str(content),
            Fragment::Link { text, .. } => out.push_str(&strip_markdown(text)),
            Fragment::Formatted { content, .. } => out.push_str(&strip_markdown(content)),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markdown_plain() {
        assert_eq!(strip_markdown("hello world"), "hello world");
    }

    #[test]
    fn test_strip_markdown_bold() {
        assert_eq!(strip_markdown("*bold*"), "bold");
    }

    #[test]
    fn test_strip_markdown_nested() {
        assert_eq!(strip_markdown("*bold _italic_ bold*"), "bold italic bold");
    }

    #[test]
    fn test_strip_markdown_link() {
        assert_eq!(
            strip_markdown("[click here](https://example.com)"),
            "click here"
        );
    }

    #[test]
    fn test_strip_markdown_code_block() {
        assert_eq!(strip_markdown("```rust\nfn main() {}\n```"), "fn main() {}");
    }

    #[test]
    fn test_strip_markdown_inline_code() {
        assert_eq!(strip_markdown("`some code`"), "some code");
    }

    #[test]
    fn test_strip_markdown_escaped() {
        assert_eq!(strip_markdown(r"hello \*world\*"), "hello *world*");
    }
}
