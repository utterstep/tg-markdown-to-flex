def tg_markdown_to_flex(text: str) -> str:
    """Convert Telegram MarkdownV2 text to a LINE Flex Message JSON string.

    Parses the following Telegram MarkdownV2 constructs and maps them to
    LINE Flex Message components:

    - ``*bold*`` → span with ``weight: "bold"``
    - ``_italic_`` → span with ``style: "italic"``
    - ``__underline__`` → span with ``decoration: "underline"``
    - ``~strikethrough~`` → span with ``decoration: "line-through"``
    - ``||spoiler||`` → span with near-white color
    - `` `inline code` `` → span with red color, small size
    - ```` ```code block```` ```` → separate text component with red color
    - ``[text](url)`` → blue underlined span + footer URI button
    """
    ...
