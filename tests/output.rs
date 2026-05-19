//! Direct tests on serializer output strings.
//!
//! These tests don't compile the generated code; they only assert the exact
//! byte sequence emitted by `Uneval::uneval`. The batch_run-driven fixture
//! tests in `generate_and_use.rs` cover the end-to-end "generate then
//! re-evaluate as Rust source" path.

use std::collections::{BTreeMap, HashMap};
use uneval::Uneval;

fn ser<T: Uneval + ?Sized>(value: &T) -> String {
    uneval::to_string(value).unwrap()
}

#[derive(Uneval)]
struct WithStaticStr {
    name: &'static str,
    count: i32,
}

#[derive(Uneval)]
struct WithString {
    name: String,
}

#[test]
fn static_str_field_has_no_into_wrap() {
    let out = ser(&WithStaticStr { name: "hello", count: 7 });
    assert_eq!(out, r#"WithStaticStr { name: "hello", count: 7i32 }"#);
    assert!(!out.contains(".into()"));
    assert!(!out.contains(".to_string()"));
}

#[test]
fn string_field_emits_to_string() {
    let out = ser(&WithString {
        name: "hello".into(),
    });
    assert_eq!(out, r#"WithString { name: "hello".to_string() }"#);
}

#[test]
fn const_compatible_when_only_const_compatible_types_used() {
    const VALUE: WithStaticStr = WithStaticStr {
        name: "hello",
        count: 7i32,
    };
    assert_eq!(ser(&VALUE), r#"WithStaticStr { name: "hello", count: 7i32 }"#);
}

#[test]
fn printable_unicode_is_preserved_verbatim() {
    let out = ser(&WithStaticStr {
        name: "矛盾 and ❤",
        count: 0,
    });
    assert_eq!(
        out,
        r#"WithStaticStr { name: "矛盾 and ❤", count: 0i32 }"#
    );
}

#[test]
fn required_string_escapes_only() {
    let out = ser(&WithStaticStr {
        name: "a\\b\"c\nd\r\te\0f",
        count: 0,
    });
    assert_eq!(
        out,
        r#"WithStaticStr { name: "a\\b\"c\nd\r\te\0f", count: 0i32 }"#
    );
}

#[derive(Uneval)]
struct WithChar {
    c: char,
}

#[test]
fn char_uses_single_quote_escape() {
    assert_eq!(ser(&WithChar { c: '\'' }), r"WithChar { c: '\'' }");
    assert_eq!(ser(&WithChar { c: '"' }), r#"WithChar { c: '"' }"#);
    assert_eq!(ser(&WithChar { c: '\\' }), r"WithChar { c: '\\' }");
    assert_eq!(ser(&WithChar { c: '矛' }), r"WithChar { c: '矛' }");
}

#[derive(Uneval)]
struct RawIdent {
    r#type: i32,
    r#match: bool,
    normal: i32,
}

#[test]
fn keyword_field_names_get_raw_prefix() {
    let v = RawIdent {
        r#type: 1,
        r#match: true,
        normal: 3,
    };
    assert_eq!(
        ser(&v),
        "RawIdent { r#type: 1i32, r#match: true, normal: 3i32 }"
    );
}

mod renames {
    //! The fundamental rename_all bug: with serde-based serialization, a struct
    //! that has `#[serde(rename_all = "kebab-case")]` would emit
    //! `Foo { some-field: ... }` — invalid Rust. The derive-based path uses the
    //! original Rust identifier from the type definition, so it is immune.

    use super::ser;
    use uneval::Uneval;

    // Yes, both serde rename_all AND uneval derive coexist — the rename only
    // affects serde's data-format serialization, not what uneval emits.
    #[derive(Uneval, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    struct ServerConfig {
        server_host: String,
        server_port: u16,
        tls_enabled: bool,
    }

    #[test]
    fn rename_all_does_not_affect_uneval_output() {
        let v = ServerConfig {
            server_host: "localhost".into(),
            server_port: 8080,
            tls_enabled: true,
        };
        let out = ser(&v);
        assert!(out.contains("server_host"));
        assert!(out.contains("server_port"));
        assert!(out.contains("tls_enabled"));
        assert!(!out.contains("server-host"));
    }

    #[derive(Uneval, serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    enum Status {
        Active,
        Inactive,
    }

    #[test]
    fn enum_rename_all_does_not_affect_uneval_output() {
        assert_eq!(ser(&Status::Active), "Status::Active");
        assert_eq!(ser(&Status::Inactive), "Status::Inactive");
    }
}

#[derive(Uneval)]
enum Mode {
    Off,
    Tuple(u32, String),
    Struct { x: i32, y: i32 },
}

#[test]
fn enum_unit_variant() {
    assert_eq!(ser(&Mode::Off), "Mode::Off");
}

#[test]
fn enum_tuple_variant() {
    assert_eq!(
        ser(&Mode::Tuple(5, "hi".into())),
        r#"Mode::Tuple(5u32, "hi".to_string())"#
    );
}

#[test]
fn enum_struct_variant() {
    assert_eq!(
        ser(&Mode::Struct { x: 1, y: 2 }),
        "Mode::Struct { x: 1i32, y: 2i32 }"
    );
}

#[derive(Uneval)]
struct UnitStruct;

#[derive(Uneval)]
struct TupleStruct(i32, i32);

#[test]
fn unit_struct() {
    assert_eq!(ser(&UnitStruct), "UnitStruct");
}

#[test]
fn tuple_struct() {
    assert_eq!(ser(&TupleStruct(1, 2)), "TupleStruct(1i32, 2i32)");
}

#[test]
fn collections() {
    let v: Vec<u32> = vec![1, 2, 3];
    assert_eq!(ser(&v), "::std::vec![1u32, 2u32, 3u32]");

    let empty: Vec<u32> = vec![];
    assert_eq!(ser(&empty), "::std::vec![]");

    let m: BTreeMap<String, i32> = BTreeMap::from([("a".to_string(), 1)]);
    assert_eq!(
        ser(&m),
        r#"::std::collections::BTreeMap::from([("a".to_string(), 1i32)])"#
    );

    let m: HashMap<i32, &'static str> = HashMap::from([(1, "x")]);
    assert_eq!(
        ser(&m),
        r#"::std::collections::HashMap::from([(1i32, "x")])"#
    );
}

#[test]
fn arrays_and_tuples() {
    let arr: [u8; 3] = [1, 2, 3];
    assert_eq!(ser(&arr), "[1u8, 2u8, 3u8]");

    let t: (i32, &'static str, bool) = (1, "x", true);
    assert_eq!(ser(&t), r#"(1i32, "x", true,)"#);
}

#[test]
fn option_and_result() {
    let v: Option<i32> = None;
    assert_eq!(ser(&v), "::std::option::Option::None");

    let v: Option<i32> = Some(5);
    assert_eq!(ser(&v), "::std::option::Option::Some(5i32)");
}

#[derive(Uneval)]
struct Generic<T> {
    value: T,
}

#[test]
fn generic_struct() {
    let v = Generic { value: 42i32 };
    assert_eq!(ser(&v), "Generic { value: 42i32 }");

    let v = Generic { value: "hi" };
    assert_eq!(ser(&v), r#"Generic { value: "hi" }"#);
}

#[cfg(feature = "url")]
mod url_feature {
    use super::ser;
    use uneval::Uneval;
    use url::Url;

    #[test]
    fn url_emits_parse_unwrap() {
        let u = Url::parse("https://example.com/path?q=1").unwrap();
        assert_eq!(
            ser(&u),
            r#"::url::Url::parse("https://example.com/path?q=1").unwrap()"#
        );
    }

    #[test]
    fn url_round_trips_through_parse() {
        // Url::parse normalizes IDN hosts to punycode and percent-encodes
        // non-ASCII path bytes. The serialized form matches `Url::as_str()`,
        // so the generated code re-parses to an equal value.
        let u = Url::parse("https://例え.テスト/路径#frag").unwrap();
        let reconstructed: Url = ::url::Url::parse(u.as_str()).unwrap();
        assert_eq!(reconstructed, u);
        assert!(ser(&u).starts_with(r#"::url::Url::parse("https://xn--"#));
    }

    #[derive(Uneval)]
    struct Endpoint {
        name: String,
        url: Url,
    }

    #[test]
    fn url_field_inside_a_derived_struct() {
        let v = Endpoint {
            name: "demo".into(),
            url: Url::parse("https://example.com/").unwrap(),
        };
        assert_eq!(
            ser(&v),
            r#"Endpoint { name: "demo".to_string(), url: ::url::Url::parse("https://example.com/").unwrap() }"#
        );
    }
}
