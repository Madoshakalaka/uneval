//! Embed Rust values as Rust source code at build time.
//!
//! `uneval` lets a build script take a runtime value and write out the *Rust
//! source code* that reconstructs it. The output is then `include!`d in the
//! main crate, so the value is rebuilt at compile time with no parsing,
//! deserializing, or `serde` overhead. For large embedded data this can shave
//! a significant amount of startup work, and it lets you drop the serializer
//! from the runtime dependency tree entirely.
//!
//! # Quick start
//!
//! ```
//! use uneval::Uneval;
//!
//! #[derive(Uneval)]
//! struct Config {
//!     name: String,
//!     count: u32,
//! }
//!
//! let cfg = Config { name: "demo".into(), count: 7 };
//! let code = uneval::to_string(&cfg).unwrap();
//! assert_eq!(code, r#"Config { name: "demo".to_string(), count: 7u32 }"#);
//! ```
//!
//! In a build script you would typically write to a file and `include!` it:
//!
//! ```ignore
//! // build.rs
//! uneval::to_out_dir(&cfg, "config.rs").unwrap();
//! ```
//!
//! ```ignore
//! // src/lib.rs
//! let cfg: Config = include!(concat!(env!("OUT_DIR"), "/config.rs"));
//! ```
//!
//! # How it works
//!
//! Each type's [`Uneval`] impl writes the constructor expression for *its own
//! type*. That gives type-aware output without any global mode switch:
//!
//! - `i32` writes `42i32`
//! - `String` writes `"hello".to_string()`
//! - `&'static str` writes `"hello"` (so it works inside `const`/`static` items)
//! - `Vec<T>` writes `::std::vec![…]`
//! - `Cow<'static, str>` writes `::std::borrow::Cow::Borrowed("hello")`
//! - `Option<T>` writes `::std::option::Option::Some(…)` or `…::None`
//! - `HashMap<K, V>` writes `::std::collections::HashMap::from([(k, v), …])`
//!
//! Because the derive walks your type definition at macro expansion time, it
//! uses the *original Rust* field and variant names. `#[serde(rename_all)]`
//! and similar attributes have no effect on what `uneval` emits, since the
//! generated code is consumed by the Rust compiler, not by a `serde` data
//! format.
//!
//! # Implementing `Uneval` manually
//!
//! `#[derive(Uneval)]` handles structs and enums. For a type the derive
//! cannot reach (foreign types, types with constructor invariants), implement
//! the trait directly:
//!
//! ```
//! use std::io::{self, Write};
//! use uneval::Uneval;
//!
//! struct PortNumber(u16);
//!
//! impl Uneval for PortNumber {
//!     fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
//!         write!(w, "PortNumber({}u16)", self.0)
//!     }
//! }
//! ```
//!
//! # Limitations
//!
//! 1. All the types named in the generated expression must be in scope at the
//!    `include!` site. The serializer only knows the type's last identifier
//!    segment, never its fully qualified path.
//! 2. Private fields cannot be constructed from outside the defining module,
//!    so types with private fields are only embeddable from inside that
//!    module (or via a public constructor — see the manual impl example above).
//! 3. Foreign types that don't have an `Uneval` impl can be wrapped in a
//!    newtype that does.

mod funcs;
mod helpers;
mod impls;

pub use funcs::{to_file, to_out_dir, to_string, write};
pub use uneval_derive::Uneval;

/// Types that can be written out as the Rust source code that constructs them.
///
/// Use `#[derive(Uneval)]` on your own types. The crate ships blanket impls
/// for all primitive types, `String`, `&str`, `Cow<'_, str>`, `Box<T>`,
/// references, `Option`, `Result`, `Vec`, slices, arrays, `HashMap`/`BTreeMap`,
/// `HashSet`/`BTreeSet`, `VecDeque`, and tuples up to arity 12.
pub trait Uneval {
    /// Write the Rust source code that, when evaluated in the right type
    /// context, reconstructs `self`. The output is always valid UTF-8.
    fn uneval(&self, w: &mut dyn std::io::Write) -> std::io::Result<()>;
}
