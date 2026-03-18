/// Telegram MarkdownV2 special characters that must be escaped in regular text.
///
/// Source of truth: <https://core.telegram.org/bots/api#markdownv2-style>
const TG_SPECIAL_CHARS: &[char] = &[
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!', '\\',
];

/// O(1) lookup table built at compile time from [`TG_SPECIAL_CHARS`].
const TG_SPECIAL: [bool; 128] = {
    let mut table = [false; 128];
    let mut i = 0;
    while i < TG_SPECIAL_CHARS.len() {
        table[TG_SPECIAL_CHARS[i] as usize] = true;
        i += 1;
    }
    table
};

/// Returns `true` if `c` is a Telegram MarkdownV2 special character.
fn is_tg_special(c: char) -> bool {
    let code = c as u32;
    code < 128 && TG_SPECIAL[code as usize]
}

// ---------------------------------------------------------------------------
// Finding helpers (work with slices, return relative offsets)
// ---------------------------------------------------------------------------

/// Find the end of a code block. `after_opening` starts right after the opening `` ``` ``.
/// Returns the byte length consumed (including the closing `` ``` ``), or `None`.
fn find_code_block_end(after_opening: &str) -> Option<usize> {
    let newline_pos = after_opening.find('\n')?;
    let mut search_from = newline_pos;
    while search_from < after_opening.len() {
        let pos = after_opening[search_from..].find("\n```")?;
        let end = search_from + pos + 4; // \n + ```
        if end >= after_opening.len() || after_opening[end..].starts_with('\n') {
            return Some(end);
        }
        search_from += pos + 1;
    }
    None
}

/// Find the position of a closing delimiter in `content`.
/// Returns the byte offset relative to `content`, or `None`.
///
/// Skips over:
/// - already-escaped characters (`\X`)
/// - inline code spans (`` `...` ``)
/// - code blocks (`` ```...``` ``)
fn find_closing(content: &str, delim: &str) -> Option<usize> {
    let mut i = 0;

    while i < content.len() {
        let ch = content[i..].chars().next().unwrap();

        // Skip already-escaped characters
        if ch == '\\'
            && let Some(next_ch) = content.get(i + 1..).and_then(|s| s.chars().next())
            && is_tg_special(next_ch)
        {
            i += 1 + next_ch.len_utf8();
            continue;
        }

        // Skip code blocks
        if content[i..].starts_with("```")
            && let Some(end) = find_code_block_end(&content[i + 3..])
        {
            i += 3 + end;
            continue;
        }

        // Skip inline code
        if ch == '`'
            && let Some(pos) = content[i + 1..].find('`')
        {
            i += pos + 2; // past both backticks
            continue;
        }

        // Check for closing delimiter
        if content[i..].starts_with(delim) {
            return Some(i);
        }

        i += ch.len_utf8();
    }

    None
}

// ---------------------------------------------------------------------------
// Inline formatting delimiter table
// ---------------------------------------------------------------------------

/// Edge-case guard for a formatting delimiter.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DelimiterGuard {
    /// No special handling.
    None,
    /// Reject if the opening is immediately followed by an extra copy of the
    /// delimiter's first character.  Prevents `__` from greedily matching `___`
    /// (underline eating into italic).
    RejectTripled,
    /// Reject if the closing delimiter is adjacent to another copy of the same
    /// character.  Prevents single `_` (italic) from matching a `_` that is
    /// part of `__` (underline).
    RejectDoubledClose,
}

struct InlineDelimiter {
    delim: &'static str,
    guard: DelimiterGuard,
}

impl InlineDelimiter {
    /// Returns `true` if the opening context rejects this match.
    fn open_rejected(&self, after_open: &str) -> bool {
        match self.guard {
            DelimiterGuard::RejectTripled => after_open.starts_with(&self.delim[..1]),
            _ => false,
        }
    }

    /// Returns `true` if the closing position should be rejected.
    fn close_rejected(&self, after_open: &str, close_pos: usize) -> bool {
        match self.guard {
            DelimiterGuard::RejectDoubledClose => {
                let dc = self.delim.as_bytes()[0];
                let len = self.delim.len();
                after_open.as_bytes().get(close_pos + len) == Some(&dc)
                    || (close_pos > 0 && after_open.as_bytes().get(close_pos - 1) == Some(&dc))
            }
            _ => false,
        }
    }
}

/// Inline formatting delimiters, checked **in order**.
///
/// Multi-character delimiters must precede their single-character subsets
/// (e.g. `||` before `|`, `__` before `_`).
const INLINE_DELIMITERS: &[InlineDelimiter] = &[
    InlineDelimiter {
        delim: "||",
        guard: DelimiterGuard::None,
    }, // spoiler
    InlineDelimiter {
        delim: "__",
        guard: DelimiterGuard::RejectTripled,
    }, // underline
    InlineDelimiter {
        delim: "*",
        guard: DelimiterGuard::None,
    }, // bold
    InlineDelimiter {
        delim: "_",
        guard: DelimiterGuard::RejectDoubledClose,
    }, // italic
    InlineDelimiter {
        delim: "~",
        guard: DelimiterGuard::None,
    }, // strikethrough
];

// ---------------------------------------------------------------------------
// Fragment: parsed piece of the input
// ---------------------------------------------------------------------------

/// A parsed fragment of the input text.
pub(crate) enum Fragment<'a> {
    /// Already-escaped character (e.g., `\*`), pass through verbatim.
    Escaped(char),
    /// Code block content (between `` ``` `` markers).
    CodeBlock(&'a str),
    /// Inline code content (between `` ` `` markers).
    InlineCode(&'a str),
    /// Link with text and URL.
    Link { text: &'a str, url: &'a str },
    /// Formatted text with delimiter (e.g., `*bold*`).
    Formatted {
        delim: &'static str,
        content: &'a str,
    },
    /// Plain character.
    Plain(char),
}

// ---------------------------------------------------------------------------
// Fragment parsers — each returns `Some` and advances `input` on success,
// or returns `None` leaving `input` unchanged.
// ---------------------------------------------------------------------------

fn try_escaped_char<'a>(input: &mut &'a str) -> Option<Fragment<'a>> {
    let rest = *input;
    let mut chars = rest.chars();
    if chars.next()? != '\\' {
        return None;
    }
    let next = chars.next().filter(|c| is_tg_special(*c))?;
    *input = &rest[1 + next.len_utf8()..];
    Some(Fragment::Escaped(next))
}

fn try_code_block<'a>(input: &mut &'a str) -> Option<Fragment<'a>> {
    let rest = *input;
    let after_opening = rest.strip_prefix("```")?;
    let end = find_code_block_end(after_opening)?;
    let content = &after_opening[..end - 3]; // everything before closing ```
    *input = &after_opening[end..];
    Some(Fragment::CodeBlock(content))
}

fn try_inline_code<'a>(input: &mut &'a str) -> Option<Fragment<'a>> {
    let rest = *input;
    let after_backtick = rest.strip_prefix('`')?;
    let close = after_backtick.find('`')?;
    let content = &after_backtick[..close];
    *input = &after_backtick[close + 1..];
    Some(Fragment::InlineCode(content))
}

fn try_link<'a>(input: &mut &'a str) -> Option<Fragment<'a>> {
    let rest = *input;
    let after_bracket = rest.strip_prefix('[')?;

    let bracket_close = find_closing(after_bracket, "]")?;
    let after_text = after_bracket[bracket_close + 1..].strip_prefix('(')?;
    let paren_close = after_text.find(')')?;

    let text = &after_bracket[..bracket_close];
    let url = &after_text[..paren_close];
    *input = &after_text[paren_close + 1..];
    Some(Fragment::Link { text, url })
}

fn try_formatting<'a>(input: &mut &'a str) -> Option<Fragment<'a>> {
    let rest = *input;

    for d in INLINE_DELIMITERS {
        if !rest.starts_with(d.delim) {
            continue;
        }

        let len = d.delim.len();
        let after_open = &rest[len..];

        if d.open_rejected(after_open) {
            continue;
        }

        let Some(close) = find_closing(after_open, d.delim) else {
            continue;
        };

        if d.close_rejected(after_open, close) {
            continue;
        }

        let content = &after_open[..close];
        *input = &after_open[close + len..];
        return Some(Fragment::Formatted {
            delim: d.delim,
            content,
        });
    }

    None
}

/// Parse the next fragment from `input`, advancing past it.
pub(crate) fn next_fragment<'a>(input: &mut &'a str) -> Fragment<'a> {
    if let Some(f) = try_escaped_char(input) {
        return f;
    }
    if let Some(f) = try_code_block(input) {
        return f;
    }
    if let Some(f) = try_inline_code(input) {
        return f;
    }
    if let Some(f) = try_link(input) {
        return f;
    }
    if let Some(f) = try_formatting(input) {
        return f;
    }

    let ch = input.chars().next().unwrap();
    *input = &input[ch.len_utf8()..];
    Fragment::Plain(ch)
}
