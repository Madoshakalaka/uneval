//! Direct tests on serializer output strings.
//!
//! These tests do not need to compile the generated code; they only assert that the
//! exact byte sequence emitted by the serializer matches expectations. The
//! batch_run-based integration tests in `generate_and_use.rs` cover the
//! end-to-end "generate and re-evaluate" path.

use serde::Serialize;
use uneval::{ser::Uneval, StringMode};

fn ser_default<T: Serialize>(value: &T) -> String {
    uneval::to_string(value).unwrap()
}

fn ser_literal<T: Serialize>(value: &T) -> String {
    let mut buf = Vec::new();
    value
        .serialize(&mut Uneval::new(&mut buf).string_mode(StringMode::Literal))
        .unwrap();
    String::from_utf8(buf).unwrap()
}

#[derive(Serialize)]
struct OneStr {
    name: &'static str,
}

#[test]
fn default_mode_wraps_strings_with_into() {
    let out = ser_default(&OneStr { name: "hello" });
    assert_eq!(out, r#"OneStr {name: "hello".into()}"#);
}

#[test]
fn literal_mode_emits_bare_string_literal() {
    let out = ser_literal(&OneStr { name: "hello" });
    assert_eq!(out, r#"OneStr {name: "hello"}"#);
    assert!(!out.contains(".into()"));
}

#[test]
fn literal_mode_compiles_in_const_context() {
    let out = ser_literal(&OneStr { name: "static" });
    assert_eq!(out, r#"OneStr {name: "static"}"#);
}

#[test]
fn printable_unicode_is_preserved_verbatim() {
    let out = ser_default(&OneStr { name: "矛盾 and ❤" });
    assert_eq!(out, r#"OneStr {name: "矛盾 and ❤".into()}"#);

    let out = ser_literal(&OneStr { name: "矛盾 and ❤" });
    assert_eq!(out, r#"OneStr {name: "矛盾 and ❤"}"#);
}

#[test]
fn required_escapes_only() {
    let out = ser_literal(&OneStr {
        name: "a\\b\"c\nd\r\te\0f",
    });
    assert_eq!(out, r#"OneStr {name: "a\\b\"c\nd\r\te\0f"}"#);
}

#[test]
fn control_char_below_space_uses_unicode_escape() {
    let out = ser_literal(&OneStr { name: "\x01\x07" });
    assert_eq!(out, r#"OneStr {name: "\u{1}\u{7}"}"#);
}

#[derive(Serialize)]
struct OneChar {
    c: char,
}

#[test]
fn char_uses_single_quote_escapes() {
    let out = ser_default(&OneChar { c: '\'' });
    assert_eq!(out, r"OneChar {c: '\''}");

    let out = ser_default(&OneChar { c: '"' });
    assert_eq!(out, r#"OneChar {c: '"'}"#);

    let out = ser_default(&OneChar { c: '\\' });
    assert_eq!(out, r"OneChar {c: '\\'}");

    let out = ser_default(&OneChar { c: '矛' });
    assert_eq!(out, r"OneChar {c: '矛'}");
}

#[derive(Serialize)]
struct RawIdent {
    r#type: i32,
    r#match: bool,
    r#use: u8,
    normal: i32,
}

#[test]
fn keyword_field_names_get_raw_prefix() {
    let v = RawIdent {
        r#type: 1,
        r#match: true,
        r#use: 2,
        normal: 3,
    };
    let out = ser_default(&v);
    assert_eq!(
        out,
        "RawIdent {r#type: 1i32,r#match: true,r#use: 2u8,normal: 3i32}"
    );
}

#[derive(Serialize)]
enum WithKeyword {
    Normal,
    r#Type,
    r#Match(i32),
}

#[test]
fn keyword_variant_names_get_raw_prefix() {
    assert_eq!(ser_default(&WithKeyword::Normal), "WithKeyword::Normal");
    let out = ser_default(&WithKeyword::r#Type);
    assert_eq!(out, "WithKeyword::Type");

    let out = ser_default(&WithKeyword::r#Match(5));
    assert_eq!(out, "WithKeyword::Match(5i32)");
}

#[test]
fn standalone_lowercase_keyword_variant_gets_raw_prefix() {
    #[derive(Serialize)]
    #[allow(non_camel_case_types)]
    enum LowerKw {
        r#fn,
    }
    let out = ser_default(&LowerKw::r#fn);
    assert_eq!(out, "LowerKw::r#fn");
}
