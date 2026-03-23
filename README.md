# tg-markdown-to-flex

Convert Telegram MarkdownV2 text to [LINE Flex Messages](https://developers.line.biz/en/docs/messaging-api/using-flex-messages/).

Available as both a Rust crate and a Python package (via [PyO3](https://pyo3.rs/)).

## What it does

Takes a string with Telegram MarkdownV2 formatting and produces a LINE Flex Message JSON — a bubble with styled text spans, code blocks, and link buttons.

### Formatting mapping

| Telegram MarkdownV2 | LINE Flex |
|---|---|
| `*bold*` | span `weight: "bold"` |
| `_italic_` | span `style: "italic"` |
| `__underline__` | span `decoration: "underline"` |
| `~strikethrough~` | span `decoration: "line-through"` |
| `` `inline code` `` | span with red color, small size |
| ` ```code block``` ` | separate text component, red/small |
| `\|\|spoiler\|\|` | span with near-white color |
| `[text](url)` | link button (see below) |

### Smart link dedup

By default, links at the end of a line are rendered as a button in the message body (with a separator and grey background), avoiding duplication. Links in the middle of text still get an inline blue/underlined span plus a footer button.

This behavior is configurable — see the options below.

## Python

Requires Python 3.13+.

### Installation

```bash
uv add tg-markdown-to-flex

# or
pip install tg-markdown-to-flex
```

### Usage

```python
import json
from tg_markdown_to_flex import tg_markdown_to_flex

flex_json = tg_markdown_to_flex("Hello *bold* and _italic_ with [a link](https://example.com)")
message = json.loads(flex_json)

# Send via LINE Messaging API
# line_bot_api.push_message(to, FlexMessage(alt_text=message["altText"], contents=message["contents"]))
```

The function returns a JSON string representing a complete Flex Message (type `"flex"` with `altText` and `contents`).

#### Options

```python
tg_markdown_to_flex(
    text,
    *,
    standalone_links_as_buttons=True,  # dedup trailing links as body buttons
    decorate_links=True,               # blue/underline styling on inline links
)
```

### Type checking

The package ships with PEP 561 type stubs.

## Rust

### Installation

```toml
[dependencies]
tg-markdown-to-flex = "0.3.0"
```

### Usage

```rust
use tg_markdown_to_flex::{tg_markdown_to_flex, tg_markdown_to_flex_json};

// Get a FlexMessage struct
let message = tg_markdown_to_flex("Hello *world*");

// Or get JSON directly
let json = tg_markdown_to_flex_json("Hello *world*");
```

With options:

```rust
use tg_markdown_to_flex::{tg_markdown_to_flex_with_options, ConvertOptions};

let options = ConvertOptions {
    standalone_links_as_buttons: false,
    decorate_links: false,
};
let message = tg_markdown_to_flex_with_options("Hello *world*", &options);
```

## Testing

```bash
cargo nextest run
```

## License

MIT
