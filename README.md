# uneval

Embed Rust values into your binary as the *Rust source code* that reconstructs them.

### Why?

Inspired by [this Stack Overflow question](https://stackoverflow.com/questions/58359340/deserialize-file-using-serde-json-at-compile-time). If you have data in some human-readable format (JSON, TOML, ...) and want it baked into the binary as if you had written it out by hand, `uneval` lets your build script translate the runtime value into Rust source code. The main crate `include!`s that source, so the data is reconstructed at compile time with no parser, no deserializer, and no `serde` at runtime.

### How

Derive `Uneval` on your types, then call `uneval::to_file` (or `to_out_dir`, `to_string`, `write`) from a build script:

```rust
use uneval::Uneval;

#[derive(Uneval)]
struct Config {
    name: String,
    port: u16,
    tls: bool,
}

// build.rs
let cfg = Config { name: "demo".into(), port: 8080, tls: true };
uneval::to_out_dir(&cfg, "config.rs").unwrap();
```

```rust
// src/lib.rs
let cfg: Config = include!(concat!(env!("OUT_DIR"), "/config.rs"));
```

### How does it work?

Each type's `Uneval` impl writes the constructor expression for its own type, so output is type-aware with no global mode switch:

| Type | Emitted |
| ---- | ------- |
| `i32`, `u8`, ... | `42i32`, `255u8` |
| `&'static str` | `"hello"` (bare literal, const-compatible) |
| `String` | `"hello".to_string()` |
| `Cow<'static, str>` | `::std::borrow::Cow::Borrowed("hello")` |
| `Vec<T>` | `::std::vec![...]` |
| `HashMap<K, V>` | `::std::collections::HashMap::from([(k, v), ...])` |
| `Option<T>` | `::std::option::Option::Some(...)` / `...::None` |
| `(T1, T2, ...)` | `(t1, t2, ...)` |

The crate ships blanket impls for primitives, `String`, `&str`, `Cow<'_, str>`, `Box<T>`, references, `Option`, `Result`, `Vec`, `VecDeque`, slices, arrays, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, and tuples up to arity 12.

### Why a custom derive instead of serde?

`serde`'s `Serialize` only ever hands the serializer the *renamed* field/variant name (after `rename_all`, `rename`, etc.), never the original Rust identifier. That makes it impossible to emit a struct or enum literal that the Rust compiler will accept whenever the rename changes the name. The custom derive walks your type definition at macro expansion time, so it uses the Rust identifiers directly and is immune to `serde` rename attributes.

This also means there is no `.into()` clutter: each type emits the constructor it wants, so `&'static str` fields stay as bare literals (which lets the generated code go inside `const`/`static` items), while `String` fields explicitly call `.to_string()`.

### Implementing `Uneval` manually

For foreign types or types with private fields and a public constructor, write the impl by hand:

```rust
use std::io::{self, Write};
use uneval::Uneval;

struct PortNumber(u16);

impl Uneval for PortNumber {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write!(w, "PortNumber({}u16)", self.0)
    }
}
```

### Limitations

1. Every type named in the generated expression must be in scope at the `include!` site. The derive only knows the type's last identifier segment, never its full path.
2. Types with private fields can only be embedded from inside the defining module (or via a public constructor exposed by the manual impl approach above).
3. Foreign types without an `Uneval` impl have to be wrapped in a newtype.

### Testing

`uneval` uses [`batch_run`](https://crates.io/crates/batch_run) to drive end-to-end tests: each fixture in `test_fixtures/data.toml` generates a `-main.rs` that emits source via `uneval::to_file`, then a `-user.rs` that `include!`s the generated source and asserts it equals the original value. Some fixtures use `const_compat = true` to also verify the generated code parses inside a `const` item.

# License

MIT
