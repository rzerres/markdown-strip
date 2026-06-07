//! # markdown-strip
//!
//! Reduce Markdown text to plain text. Conservative — keeps semantic
//! content intact, drops only formatting markers. Intended for piping
//! LLM output into TTS, keyword matching, or analytics.
//!
//! Handles: ATX headers (`# `, `## ` …), bold/italic (`**x**`, `*x*`,
//! `__x__`, `_x_`), inline code (backtick spans), fenced code blocks,
//! links (`[text](url)` → `text`), images (`![alt](url)` → `alt`),
//! blockquote markers (`> `), and bullet/number list markers.
//!
//! ## Example
//!
//! ```
//! use markdown_strip::strip_markdown;
//! let md = "## Hello\n\n**bold** and *italic* with `code` and [a link](https://x).";
//! let plain = strip_markdown(md);
//! assert_eq!(plain, "Hello\n\nbold and italic with code and a link.");
//! ```

#![deny(missing_docs)]

/// Strip Markdown formatting from `s`.
pub fn strip_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_fence = false;

    for line in s.lines() {
        // Fenced code blocks: skip the fences themselves; keep the body
        // as-is (LLM tool output is often code we want to preserve).
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        let stripped = strip_line(line);
        out.push_str(&stripped);
        out.push('\n');
    }

    // Trim the final pushed newline if input didn't end with one.
    if !s.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

fn strip_line(line: &str) -> String {
    let mut s = line.to_string();

    // ATX headers: leading `#{1,6}\s+`
    s = strip_atx_header(&s);

    // Blockquote: leading `>` optionally followed by space.
    s = strip_blockquote(&s);

    // Bullet/number list markers at the start.
    s = strip_list_marker(&s);

    // Inline tokens (links, images, bold, italic, code).
    s = strip_inline(&s);

    s
}

fn strip_atx_header(s: &str) -> String {
    let leading_ws: String = s.chars().take_while(|c| c.is_whitespace()).collect();
    let rest = &s[leading_ws.len()..];
    let mut hashes = 0;
    for c in rest.chars().take(6) {
        if c == '#' {
            hashes += 1;
        } else {
            break;
        }
    }
    if hashes > 0 && rest[hashes..].starts_with(' ') {
        format!("{leading_ws}{}", &rest[hashes + 1..])
    } else {
        s.to_string()
    }
}

fn strip_blockquote(s: &str) -> String {
    let leading_ws: String = s.chars().take_while(|c| c.is_whitespace()).collect();
    let rest = &s[leading_ws.len()..];
    if let Some(stripped) = rest.strip_prefix("> ") {
        format!("{leading_ws}{stripped}")
    } else if let Some(stripped) = rest.strip_prefix('>') {
        format!("{leading_ws}{stripped}")
    } else {
        s.to_string()
    }
}

fn strip_list_marker(s: &str) -> String {
    let leading_ws: String = s.chars().take_while(|c| c.is_whitespace()).collect();
    let rest = &s[leading_ws.len()..];
    // Unordered: -, *, +
    if let Some(stripped) = rest
        .strip_prefix("- ")
        .or(rest.strip_prefix("* "))
        .or(rest.strip_prefix("+ "))
    {
        return format!("{leading_ws}{stripped}");
    }
    // Ordered: N. or N) where N is digits
    let mut digits = 0;
    for c in rest.chars() {
        if c.is_ascii_digit() {
            digits += 1;
        } else {
            break;
        }
    }
    if digits > 0
        && rest.len() > digits + 1
        && (rest.as_bytes()[digits] == b'.' || rest.as_bytes()[digits] == b')')
        && rest.as_bytes()[digits + 1] == b' '
    {
        return format!("{leading_ws}{}", &rest[digits + 2..]);
    }
    s.to_string()
}

fn strip_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Image: ![alt](url) -> alt
        if i + 1 < bytes.len() && bytes[i] == b'!' && bytes[i + 1] == b'[' {
            if let Some((alt, end)) = parse_link(&s[i + 1..]) {
                out.push_str(&alt);
                i += 1 + end;
                continue;
            }
        }
        // Link: [text](url) -> text
        if bytes[i] == b'[' {
            if let Some((text, end)) = parse_link(&s[i..]) {
                out.push_str(&text);
                i += end;
                continue;
            }
        }
        // Inline code `code` -> code
        if bytes[i] == b'`' {
            if let Some(end_rel) = s[i + 1..].find('`') {
                out.push_str(&s[i + 1..i + 1 + end_rel]);
                i += 2 + end_rel;
                continue;
            }
        }
        // Bold **x** or __x__ → x. Italic *x* or _x_ → x. We handle the
        // bold case first so we don't strip half a bold pair as italic.
        if i + 1 < bytes.len() && (bytes[i] == b'*' && bytes[i + 1] == b'*') {
            if let Some(end_rel) = s[i + 2..].find("**") {
                out.push_str(&s[i + 2..i + 2 + end_rel]);
                i += 4 + end_rel;
                continue;
            }
        }
        if i + 1 < bytes.len() && (bytes[i] == b'_' && bytes[i + 1] == b'_') {
            if let Some(end_rel) = s[i + 2..].find("__") {
                out.push_str(&s[i + 2..i + 2 + end_rel]);
                i += 4 + end_rel;
                continue;
            }
        }
        if bytes[i] == b'*' {
            if let Some(end_rel) = s[i + 1..].find('*') {
                out.push_str(&s[i + 1..i + 1 + end_rel]);
                i += 2 + end_rel;
                continue;
            }
        }
        if bytes[i] == b'_' && is_word_boundary(bytes, i) {
            if let Some(end_rel) = s[i + 1..].find('_') {
                out.push_str(&s[i + 1..i + 1 + end_rel]);
                i += 2 + end_rel;
                continue;
            }
        }
        // Default: copy one whole UTF-8 character. `i` is always on a char
        // boundary here (every branch above advances by ASCII-aligned byte
        // counts), so we copy the full multi-byte sequence rather than
        // mangling it one byte at a time.
        let ch_len = utf8_char_len(bytes[i]);
        out.push_str(&s[i..i + ch_len]);
        i += ch_len;
    }
    out
}

fn parse_link(s: &str) -> Option<(String, usize)> {
    // s starts with '['; find ']('; then ')'.
    let bytes = s.as_bytes();
    if bytes[0] != b'[' {
        return None;
    }
    let close_text = s[1..].find("](")?;
    let after_url_off = 1 + close_text + 2;
    let close_url = s[after_url_off..].find(')')?;
    let text = s[1..1 + close_text].to_string();
    Some((text, after_url_off + close_url + 1))
}

/// Length in bytes of the UTF-8 character that starts with the leading
/// byte `b`. Returns 1 for ASCII and continuation/invalid bytes so the
/// caller always makes forward progress and stays on char boundaries.
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

fn is_word_boundary(bytes: &[u8], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let prev = bytes[i - 1];
    !prev.is_ascii_alphanumeric()
}
