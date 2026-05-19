//! Internal helpers shared by the blanket `Uneval` impls.

use std::fmt::Write as _;
use std::io::{self, Write};

/// Write a `&str` value as a Rust string literal (`"…"`), escaping only the
/// characters Rust syntax actually requires. Printable Unicode characters such
/// as `矛盾` or `❤` are written through verbatim, **not** expanded to
/// `\u{77db}\u{76fe}` the way [`char::escape_default`] would.
pub fn write_str_literal(w: &mut dyn Write, s: &str) -> io::Result<()> {
    let mut buf = String::with_capacity(s.len() + 2);
    buf.push('"');
    for c in s.chars() {
        push_escape(&mut buf, c, EscapeContext::Str);
    }
    buf.push('"');
    w.write_all(buf.as_bytes())
}

/// Write a `char` value as a Rust char literal (`'…'`), with the same escaping
/// rules as [`write_str_literal`].
pub fn write_char_literal(w: &mut dyn Write, c: char) -> io::Result<()> {
    let mut buf = String::with_capacity(4);
    buf.push('\'');
    push_escape(&mut buf, c, EscapeContext::Char);
    buf.push('\'');
    w.write_all(buf.as_bytes())
}

#[derive(Clone, Copy)]
enum EscapeContext {
    Str,
    Char,
}

fn push_escape(out: &mut String, c: char, ctx: EscapeContext) {
    match c {
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\0' => out.push_str("\\0"),
        '"' if matches!(ctx, EscapeContext::Str) => out.push_str("\\\""),
        '\'' if matches!(ctx, EscapeContext::Char) => out.push_str("\\'"),
        c if c.is_control() => {
            let _ = write!(out, "\\u{{{:x}}}", c as u32);
        }
        c => out.push(c),
    }
}
