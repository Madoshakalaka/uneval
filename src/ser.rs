//! Implementation of the Uneval serializer.

use crate::error::UnevalError;
use serde::ser;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::io::Write;

pub(crate) type SerResult = Result<(), UnevalError>;

/// Strategy for writing string values to the generated Rust code.
///
/// Serde's data model gives the serializer a `&str` but no information about the
/// destination field's type, so the serializer has to commit to a single shape for
/// every string. `StringMode` lets the caller choose which shape that is. Set it
/// via [`Uneval::string_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringMode {
    /// Wrap every string with a call to `.into()` (the historical default).
    ///
    /// Works for any destination type that implements `From<&'static str>`,
    /// including `String`, `Cow<'static, str>`, and `Box<str>`. It also still
    /// works for `&'static str` fields via the reflexive `From<T> for T` impl
    /// (though the call is then redundant).
    ///
    /// Generated code in this mode is **not** usable inside `const`/`static`
    /// items, because `Into::into` is not a const-stable trait method.
    #[default]
    IntoCall,
    /// Emit raw `&'static str` literals (`"..."`) with no conversion wrapper.
    ///
    /// Use this when the destination field type is `&'static str`, or when the
    /// generated code must be embeddable in a `const`/`static` item. The output
    /// is exactly the string literal, with only the characters that Rust
    /// requires escaped: `\\`, `\"`, and control codes.
    Literal,
}

/// Main serializer implementation.
///
/// Users are usually encouraged to use [`to_out_dir`][crate::funcs::to_out_dir] or, in special cases,
/// [`to_file`][crate::funcs::to_file], [`write`][crate::funcs::write] or [`to_string`][crate::funcs::to_string].
///
/// Construct directly when you need to override the default behavior:
///
/// ```no_run
/// use serde::Serialize;
/// use uneval::ser::{StringMode, Uneval};
///
/// #[derive(Serialize)]
/// struct Sample { name: &'static str }
///
/// let mut buf = Vec::new();
/// let mut ser = Uneval::new(&mut buf).string_mode(StringMode::Literal);
/// Sample { name: "hi" }.serialize(&mut ser).unwrap();
/// ```
pub struct Uneval<W: Write> {
    writer: W,
    inside: bool,
    string_mode: StringMode,
}

impl<W: Write> Uneval<W> {
    pub fn new(target: W) -> Self {
        Self {
            writer: target,
            inside: false,
            string_mode: StringMode::default(),
        }
    }

    /// Choose how string values are written into the generated code. See [`StringMode`].
    pub fn string_mode(mut self, mode: StringMode) -> Self {
        self.string_mode = mode;
        self
    }

    fn start_sub(&mut self) -> &mut Self {
        self.inside = false;
        self
    }

    fn comma(&mut self) -> SerResult {
        if self.inside {
            write!(self.writer, ",")?;
        }
        self.inside = true;
        Ok(())
    }

    fn serialize_item(&mut self, item: impl ser::Serialize) -> SerResult {
        self.comma()?;
        item.serialize(self)?;
        Ok(())
    }
}

impl<W: Write> ser::Serializer for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> SerResult {
        write!(self.writer, "{}", v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> SerResult {
        write!(self.writer, "{}i8", v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> SerResult {
        write!(self.writer, "{}i16", v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> SerResult {
        write!(self.writer, "{}i32", v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> SerResult {
        write!(self.writer, "{}i64", v)?;
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> SerResult {
        write!(self.writer, "{}i128", v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> SerResult {
        write!(self.writer, "{}u8", v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> SerResult {
        write!(self.writer, "{}u16", v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> SerResult {
        write!(self.writer, "{}u32", v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> SerResult {
        write!(self.writer, "{}u64", v)?;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> SerResult {
        write!(self.writer, "{}u128", v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> SerResult {
        write!(self.writer, "{}f32", v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> SerResult {
        write!(self.writer, "{}f64", v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> SerResult {
        write!(self.writer, "'{}'", escape_char(v))?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> SerResult {
        let escaped = escape_str(v);
        match self.string_mode {
            StringMode::IntoCall => write!(self.writer, "\"{}\".into()", escaped)?,
            StringMode::Literal => write!(self.writer, "\"{}\"", escaped)?,
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> SerResult {
        self.collect_seq(v)?;
        Ok(())
    }

    fn serialize_none(self) -> SerResult {
        write!(self.writer, "None")?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> SerResult
    where
        T: ?Sized + serde::Serialize,
    {
        write!(self.writer, "Some(")?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn serialize_unit(self) -> SerResult {
        write!(self.writer, "()")?;
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> SerResult {
        write!(self.writer, "{}", rust_ident(name))?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> SerResult {
        write!(self.writer, "{}::{}", rust_ident(name), rust_ident(variant))?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> SerResult
    where
        T: ?Sized + serde::Serialize,
    {
        write!(self.writer, "{}(", rust_ident(name))?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> SerResult
    where
        T: ?Sized + serde::Serialize,
    {
        write!(self.writer, "{}::{}(", rust_ident(name), rust_ident(variant))?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        write!(self.writer, "vec![")?;
        Ok(self.start_sub())
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        write!(self.writer, "{{")?;
        crate::helpers::tuple_converter(&mut self.writer, len)?;
        write!(self.writer, "convert((")?;
        Ok(self.start_sub())
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        write!(self.writer, "{}(", rust_ident(name))?;
        Ok(self.start_sub())
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        write!(self.writer, "{}::{}(", rust_ident(name), rust_ident(variant))?;
        Ok(self.start_sub())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        write!(self.writer, "vec![")?;
        Ok(self.start_sub())
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        write!(self.writer, "{} {{", rust_ident(name))?;
        Ok(self.start_sub())
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        write!(self.writer, "{}::{} {{", rust_ident(name), rust_ident(variant))?;
        Ok(self.start_sub())
    }
}

impl<W: Write> ser::SerializeSeq for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> SerResult {
        write!(self.writer, "].into_iter().collect()")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeTuple for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> SerResult {
        write!(self.writer, ")) }}")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeTupleStruct for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> SerResult {
        write!(self.writer, ")")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeTupleVariant for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> SerResult {
        write!(self.writer, ")")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeMap for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.comma()?;
        write!(self.writer, "(")?;
        key.serialize(&mut **self)?;
        write!(self.writer, ",")?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn end(self) -> SerResult {
        write!(self.writer, "].into_iter().collect()")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeStruct for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.comma()?;
        write!(self.writer, "{}: ", rust_ident(key))?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> SerResult {
        write!(self.writer, "}}")?;
        self.inside = true;
        Ok(())
    }
}
impl<W: Write> ser::SerializeStructVariant for &mut Uneval<W> {
    type Ok = ();
    type Error = UnevalError;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.comma()?;
        write!(self.writer, "{}: ", rust_ident(key))?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> SerResult {
        write!(self.writer, "}}")?;
        self.inside = true;
        Ok(())
    }
}

/// Wrap a Rust identifier in `r#` if it collides with a keyword.
///
/// Returns the original `&str` borrowed if no rewriting is needed, otherwise an
/// owned `String`. Keywords that cannot be made into raw identifiers (`crate`,
/// `self`, `Self`, `super`, `extern`) are returned unchanged: emitting them will
/// produce a clear compile error at the include site, which is preferable to a
/// silently-corrupt `r#self`.
pub(crate) fn rust_ident(name: &str) -> Cow<'_, str> {
    if needs_raw_prefix(name) {
        Cow::Owned(format!("r#{}", name))
    } else {
        Cow::Borrowed(name)
    }
}

fn needs_raw_prefix(name: &str) -> bool {
    matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "else"
            | "enum"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "static"
            | "struct"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "gen"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Escape a `&str` value for embedding inside a Rust string literal (`"..."`).
///
/// Only the characters that the Rust grammar actually requires escaping are
/// changed: backslash, double-quote, the four common control codes (`\n`, `\r`,
/// `\t`, `\0`), and other control characters (emitted as `\u{...}`). Every
/// other character — including the entire printable Unicode range — is left
/// untouched, so input like `"矛盾"` round-trips as `"矛盾"`, not `"\u{77db}\u{76fe}"`.
fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        push_escape(&mut out, c, EscapeContext::Str);
    }
    out
}

fn escape_char(c: char) -> String {
    let mut out = String::with_capacity(2);
    push_escape(&mut out, c, EscapeContext::Char);
    out
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
