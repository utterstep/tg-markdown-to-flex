mod convert;
pub mod flex;
mod parser;

pub use flex::FlexMessage;

/// Options controlling the conversion behavior.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// When `true` (the default), links at the end of a line are rendered as
    /// buttons directly in the message body instead of being duplicated as
    /// both inline styled text and a footer button.
    pub standalone_links_as_buttons: bool,
    /// When `true` (the default), non-deduped inline links are styled with
    /// blue color and underline decoration. Set to `false` to render link
    /// text with no special styling (the footer button still provides
    /// clickability).
    pub decorate_links: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            standalone_links_as_buttons: true,
            decorate_links: true,
        }
    }
}

/// Convert Telegram MarkdownV2 text to a LINE Flex Message struct.
pub fn tg_markdown_to_flex(text: &str) -> FlexMessage {
    tg_markdown_to_flex_with_options(text, &ConvertOptions::default())
}

/// Convert Telegram MarkdownV2 text to a LINE Flex Message struct with options.
pub fn tg_markdown_to_flex_with_options(text: &str, options: &ConvertOptions) -> FlexMessage {
    convert::convert(text, options)
}

/// Convert Telegram MarkdownV2 text to a LINE Flex Message JSON string.
pub fn tg_markdown_to_flex_json(text: &str) -> String {
    serde_json::to_string(&tg_markdown_to_flex(text)).expect("FlexMessage serialization failed")
}

/// Convert Telegram MarkdownV2 text to a LINE Flex Message JSON string with options.
pub fn tg_markdown_to_flex_json_with_options(text: &str, options: &ConvertOptions) -> String {
    serde_json::to_string(&tg_markdown_to_flex_with_options(text, options))
        .expect("FlexMessage serialization failed")
}

#[cfg(feature = "python")]
mod python {
    use pyo3::prelude::*;

    use super::ConvertOptions;

    /// Convert Telegram MarkdownV2 text to a LINE Flex Message JSON string.
    #[pyfunction]
    #[pyo3(signature = (text, *, standalone_links_as_buttons=true, decorate_links=true))]
    fn tg_markdown_to_flex(
        text: &str,
        standalone_links_as_buttons: bool,
        decorate_links: bool,
    ) -> String {
        let options = ConvertOptions {
            standalone_links_as_buttons,
            decorate_links,
        };
        super::tg_markdown_to_flex_json_with_options(text, &options)
    }

    #[pymodule]
    #[pyo3(name = "_core")]
    fn tg_markdown_to_flex_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(tg_markdown_to_flex, m)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn to_json(text: &str) -> serde_json::Value {
        serde_json::from_str(&tg_markdown_to_flex_json(text)).unwrap()
    }

    fn get_body_contents(val: &serde_json::Value) -> &serde_json::Value {
        &val["contents"]["body"]["contents"]
    }

    fn get_footer_contents(val: &serde_json::Value) -> &serde_json::Value {
        &val["contents"]["footer"]["contents"]
    }

    fn get_spans(val: &serde_json::Value, text_idx: usize) -> &serde_json::Value {
        &get_body_contents(val)[text_idx]["contents"]
    }

    // --- Basic structure ---

    #[test]
    fn test_basic_structure() {
        let json = to_json("hello");
        assert_eq!(json["type"], "flex");
        assert_eq!(json["altText"], "hello");
        assert_eq!(json["contents"]["type"], "bubble");
        assert_eq!(json["contents"]["body"]["type"], "box");
        assert_eq!(json["contents"]["body"]["layout"], "vertical");
    }

    #[test]
    fn test_plain_text() {
        let json = to_json("hello world");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["type"], "span");
        assert_eq!(spans[0]["text"], "hello world");
        // No styling properties
        assert_eq!(spans[0]["weight"], serde_json::Value::Null);
        assert_eq!(spans[0]["style"], serde_json::Value::Null);
    }

    // --- Formatting ---

    #[test]
    fn test_bold() {
        let json = to_json("*bold*");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "bold");
        assert_eq!(spans[0]["weight"], "bold");
    }

    #[test]
    fn test_italic() {
        let json = to_json("_italic_");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "italic");
        assert_eq!(spans[0]["style"], "italic");
    }

    #[test]
    fn test_underline() {
        let json = to_json("__underline__");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "underline");
        assert_eq!(spans[0]["decoration"], "underline");
    }

    #[test]
    fn test_strikethrough() {
        let json = to_json("~strikethrough~");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "strikethrough");
        assert_eq!(spans[0]["decoration"], "line-through");
    }

    #[test]
    fn test_spoiler() {
        let json = to_json("||spoiler||");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "spoiler");
        assert_eq!(spans[0]["color"], "#CCCCCC");
    }

    // --- Code ---

    #[test]
    fn test_inline_code() {
        let json = to_json("some `code` here");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "some ");
        assert_eq!(spans[1]["text"], "code");
        assert_eq!(spans[1]["color"], "#CC0000");
        assert_eq!(spans[1]["size"], "sm");
        assert_eq!(spans[2]["text"], " here");
    }

    #[test]
    fn test_code_block() {
        let json = to_json("before\n```rust\nfn main() {}\n```\nafter");
        let body = get_body_contents(&json);
        // First text component: "before\n"
        assert_eq!(body[0]["contents"][0]["text"], "before\n");
        // Second text component: code block
        assert_eq!(body[1]["contents"][0]["text"], "fn main() {}");
        assert_eq!(body[1]["contents"][0]["color"], "#CC0000");
        assert_eq!(body[1]["contents"][0]["size"], "sm");
        // Third text component: "\nafter"
        assert_eq!(body[2]["contents"][0]["text"], "\nafter");
    }

    // --- Nested formatting ---

    #[test]
    fn test_bold_italic() {
        let json = to_json("*bold _italic_ bold*");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "bold ");
        assert_eq!(spans[0]["weight"], "bold");
        assert_eq!(spans[1]["text"], "italic");
        assert_eq!(spans[1]["weight"], "bold");
        assert_eq!(spans[1]["style"], "italic");
        assert_eq!(spans[2]["text"], " bold");
        assert_eq!(spans[2]["weight"], "bold");
    }

    // --- Links ---

    #[test]
    fn test_link() {
        let json = to_json("Check [this](https://example.com) out");
        let spans = get_spans(&json, 0);
        // "Check "
        assert_eq!(spans[0]["text"], "Check ");
        // "this" styled as link
        assert_eq!(spans[1]["text"], "this");
        assert_eq!(spans[1]["color"], "#1689FC");
        assert_eq!(spans[1]["decoration"], "underline");
        // " out"
        assert_eq!(spans[2]["text"], " out");

        // Footer button
        let footer = get_footer_contents(&json);
        assert_eq!(footer[0]["type"], "button");
        assert_eq!(footer[0]["action"]["type"], "uri");
        assert_eq!(footer[0]["action"]["label"], "this");
        assert_eq!(footer[0]["action"]["uri"], "https://example.com");
        assert_eq!(footer[0]["style"], "link");
    }

    // --- Mixed content ---

    #[test]
    fn test_mixed_formatting() {
        let json = to_json("Hello *bold* and _italic_ world");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "Hello ");
        assert_eq!(spans[1]["text"], "bold");
        assert_eq!(spans[1]["weight"], "bold");
        assert_eq!(spans[2]["text"], " and ");
        assert_eq!(spans[3]["text"], "italic");
        assert_eq!(spans[3]["style"], "italic");
        assert_eq!(spans[4]["text"], " world");
    }

    // --- Alt text ---

    #[test]
    fn test_alt_text_stripped() {
        let json = to_json("Hello *bold* and `code`");
        assert_eq!(json["altText"], "Hello bold and code");
    }

    #[test]
    fn test_alt_text_link() {
        let json = to_json("[click here](https://example.com)");
        assert_eq!(json["altText"], "click here");
    }

    // --- Edge cases ---

    #[test]
    fn test_empty_string() {
        let json = to_json("");
        assert_eq!(json["type"], "flex");
        assert_eq!(json["altText"], "");
    }

    #[test]
    fn test_unmatched_delimiter() {
        let json = to_json("price is 5*3");
        let spans = get_spans(&json, 0);
        // Unmatched * treated as plain text
        assert_eq!(spans[0]["text"], "price is 5*3");
    }

    #[test]
    fn test_escaped_chars() {
        let json = to_json(r"hello \*world\*");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "hello *world*");
        assert_eq!(json["altText"], "hello *world*");
    }

    #[test]
    fn test_no_footer_without_links() {
        let json = to_json("no links here");
        assert_eq!(json["contents"]["footer"], serde_json::Value::Null);
    }

    // --- Standalone link dedup ---

    fn to_json_with_options(text: &str, options: &ConvertOptions) -> serde_json::Value {
        serde_json::from_str(&tg_markdown_to_flex_json_with_options(text, options)).unwrap()
    }

    #[test]
    fn test_trailing_link_deduped_to_footer() {
        let json = to_json("before\n[click here](https://example.com)");
        let body = get_body_contents(&json);
        assert_eq!(body[0]["type"], "text");
        assert_eq!(body[0]["contents"][0]["text"], "before\n");
        assert_eq!(body.as_array().unwrap().len(), 1);
        // Deduped link in footer with background
        let footer = &json["contents"]["footer"];
        assert_eq!(footer["backgroundColor"], "#F0F0F0");
        let fc = get_footer_contents(&json);
        assert_eq!(fc[0]["type"], "button");
        assert_eq!(fc[0]["action"]["label"], "click here");
        assert_eq!(fc[0]["action"]["uri"], "https://example.com");
    }

    #[test]
    fn test_trailing_link_at_start_of_input() {
        let json = to_json("[click](https://example.com)\nafter");
        let body = get_body_contents(&json);
        assert_eq!(body[0]["type"], "text");
        assert_eq!(body[0]["contents"][0]["text"], "after");
        let fc = get_footer_contents(&json);
        assert_eq!(fc[0]["action"]["label"], "click");
    }

    #[test]
    fn test_trailing_link_only() {
        let json = to_json("[click](https://example.com)");
        let fc = get_footer_contents(&json);
        assert_eq!(fc[0]["type"], "button");
        assert_eq!(fc[0]["action"]["label"], "click");
    }

    #[test]
    fn test_trailing_link_after_text_on_same_line() {
        let json = to_json("Check [this](https://example.com)");
        let body = get_body_contents(&json);
        assert_eq!(body[0]["type"], "text");
        assert_eq!(body[0]["contents"][0]["text"], "Check ");
        let fc = get_footer_contents(&json);
        assert_eq!(fc[0]["action"]["label"], "this");
    }

    #[test]
    fn test_mid_line_link_still_gets_footer() {
        // Link NOT at end of line — should still use footer
        let json = to_json("Check [this](https://example.com) out");
        let footer = get_footer_contents(&json);
        assert_eq!(footer[0]["type"], "button");
        assert_eq!(footer[0]["action"]["label"], "this");
    }

    #[test]
    fn test_dedup_disabled() {
        let options = ConvertOptions {
            standalone_links_as_buttons: false,
            ..ConvertOptions::default()
        };
        let json = to_json_with_options("before\n[click](https://example.com)", &options);
        let spans = get_spans(&json, 0);
        assert_eq!(spans[1]["text"], "click");
        assert_eq!(spans[1]["color"], "#1689FC");
        let footer = get_footer_contents(&json);
        assert_eq!(footer[0]["type"], "button");
    }

    #[test]
    fn test_mixed_trailing_and_mid_line_links() {
        let json = to_json("See [inline](https://a.com) here\n[trailing](https://b.com)");
        let body = get_body_contents(&json);
        assert_eq!(body[0]["type"], "text");
        assert_eq!(body.as_array().unwrap().len(), 1);
        // Both links end up in footer (inline first, then trailing)
        let fc = get_footer_contents(&json);
        assert_eq!(fc[0]["action"]["uri"], "https://a.com");
        assert_eq!(fc[1]["action"]["uri"], "https://b.com");
        assert_eq!(fc.as_array().unwrap().len(), 2);
    }

    // --- Link decoration ---

    #[test]
    fn test_link_decoration_disabled() {
        let options = ConvertOptions {
            standalone_links_as_buttons: false,
            decorate_links: false,
        };
        let json = to_json_with_options("Check [this](https://example.com) out", &options);
        let spans = get_spans(&json, 0);
        // Link text merges with surrounding text (no distinct styling)
        assert_eq!(spans[0]["text"], "Check this out");
        assert_eq!(spans[0]["color"], serde_json::Value::Null);
        assert_eq!(spans[0]["decoration"], serde_json::Value::Null);
        // Footer button still present
        let footer = get_footer_contents(&json);
        assert_eq!(footer[0]["type"], "button");
    }

    #[test]
    fn test_link_decoration_enabled_by_default() {
        let options = ConvertOptions {
            standalone_links_as_buttons: false,
            ..ConvertOptions::default()
        };
        let json = to_json_with_options("Check [this](https://example.com) out", &options);
        let spans = get_spans(&json, 0);
        assert_eq!(spans[1]["color"], "#1689FC");
        assert_eq!(spans[1]["decoration"], "underline");
    }

    #[test]
    fn test_cyrillic() {
        let json = to_json("Привет *мир*");
        let spans = get_spans(&json, 0);
        assert_eq!(spans[0]["text"], "Привет ");
        assert_eq!(spans[1]["text"], "мир");
        assert_eq!(spans[1]["weight"], "bold");
    }
}
