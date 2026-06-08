use markdown_strip::strip_markdown;

#[test]
fn strips_headers() {
    assert_eq!(strip_markdown("# Title"), "Title");
    assert_eq!(strip_markdown("## Subhead"), "Subhead");
}

#[test]
fn strips_bold_italic() {
    assert_eq!(strip_markdown("**bold** and *italic*"), "bold and italic");
    assert_eq!(strip_markdown("__bold__ and _italic_"), "bold and italic");
}

#[test]
fn strips_inline_code() {
    assert_eq!(
        strip_markdown("use `cargo build` to build"),
        "use cargo build to build"
    );
}

#[test]
fn strips_links_keeping_text() {
    assert_eq!(
        strip_markdown("see [docs](https://example.com)"),
        "see docs"
    );
}

#[test]
fn strips_images_keeping_alt() {
    assert_eq!(
        strip_markdown("![logo](https://example.com/x.png) heading"),
        "logo heading"
    );
}

#[test]
fn strips_list_markers() {
    let md = "- one\n- two\n- three";
    let plain = strip_markdown(md);
    assert_eq!(plain, "one\ntwo\nthree");
}

#[test]
fn strips_ordered_list() {
    let md = "1. one\n2. two\n3. three";
    let plain = strip_markdown(md);
    assert_eq!(plain, "one\ntwo\nthree");
}

#[test]
fn strips_blockquote() {
    assert_eq!(strip_markdown("> quoted line"), "quoted line");
}

#[test]
fn keeps_fenced_code_body() {
    let md = "before\n```\nlet x = 1;\n```\nafter";
    let plain = strip_markdown(md);
    assert!(plain.contains("let x = 1;"));
    assert!(!plain.contains("```"));
}

#[test]
fn does_not_strip_underscores_in_words() {
    // `snake_case_name` shouldn't be touched.
    assert_eq!(
        strip_markdown("the variable snake_case_name is set"),
        "the variable snake_case_name is set"
    );
}

#[test]
fn preserves_non_ascii_text() {
    // Multi-byte UTF-8 characters that fall through to the default copy
    // path must be preserved intact, not mangled byte-by-byte.
    assert_eq!(strip_markdown("café résumé naïve"), "café résumé naïve");
    assert_eq!(strip_markdown("hello 🦀 world"), "hello 🦀 world");
    assert_eq!(strip_markdown("## 你好\n\n世界"), "你好\n\n世界");
}

#[test]
fn strips_formatting_around_non_ascii() {
    assert_eq!(strip_markdown("**café** and *résumé*"), "café and résumé");
    assert_eq!(strip_markdown("[café](https://x) é"), "café é");
}

#[test]
fn strips_inline_markup_inside_emphasis() {
    // Markup nested inside bold/italic must also be stripped, not left raw.
    assert_eq!(strip_markdown("**[text](https://x)**"), "text");
    assert_eq!(strip_markdown("*[text](https://x)*"), "text");
    assert_eq!(strip_markdown("__[text](https://x)__"), "text");
    assert_eq!(strip_markdown("_[text](https://x)_"), "text");
    assert_eq!(strip_markdown("**`code`**"), "code");
    assert_eq!(strip_markdown("**![alt](https://x)**"), "alt");
    // Nested emphasis inside emphasis.
    assert_eq!(strip_markdown("**a *b* c**"), "a b c");
}

#[test]
fn complex_paragraph() {
    let md = "## Hello\n\n**bold** and *italic* with `code` and [a link](https://x).";
    assert_eq!(
        strip_markdown(md),
        "Hello\n\nbold and italic with code and a link."
    );
}
