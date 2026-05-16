# markdown-strip

[![crates.io](https://img.shields.io/crates/v/markdown-strip.svg)](https://crates.io/crates/markdown-strip)

Strip Markdown formatting to plain text. Conservative, fast, zero deps.
Use it to feed LLM output into TTS, keyword matching, or analytics.

```rust
use markdown_strip::strip_markdown;
let md = "## Hello\n\n**bold** and *italic* with `code` and [a link](https://x).";
assert_eq!(strip_markdown(md), "Hello\n\nbold and italic with code and a link.");
```

Handles ATX headers, bold/italic, inline code, fenced code blocks
(body preserved), links (text kept, URL dropped), images (alt kept),
blockquote markers, and bullet/number list markers.

MIT or Apache-2.0.
