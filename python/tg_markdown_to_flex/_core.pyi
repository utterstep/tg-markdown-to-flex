def tg_markdown_to_flex(
    text: str,
    *,
    standalone_links_as_buttons: bool = True,
    decorate_links: bool = True,
) -> str:
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
    - ``[text](url)`` → link button (deduped or inline + footer)

    Args:
        text: Telegram MarkdownV2 formatted text.
        standalone_links_as_buttons: When True (default), links at the end of
            a line become body buttons instead of duplicated inline + footer.
        decorate_links: When True (default), inline links are styled with
            blue color and underline. Set to False for plain text.
    """
    ...
