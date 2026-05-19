//! Convenience entry points for the most common `Uneval` workflows.

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::Uneval;

/// Write the generated Rust code to an arbitrary [`Write`] target.
pub fn write<T: Uneval + ?Sized, W: Write>(value: &T, mut target: W) -> io::Result<()> {
    value.uneval(&mut target)
}

/// Write the generated Rust code to a file at `path`, creating or truncating it.
///
/// This is the canonical way to use `uneval` from a build script:
///
/// ```no_run
/// # use uneval::Uneval;
/// # #[derive(Uneval)]
/// # struct Config { name: String }
/// # let value = Config { name: "x".into() };
/// let path: std::path::PathBuf = [
///     std::env::var("OUT_DIR").expect("OUT_DIR not set"),
///     "config.rs".into(),
/// ].iter().collect();
/// uneval::to_file(&value, path).expect("write failed");
/// ```
pub fn to_file<T: Uneval + ?Sized>(value: &T, path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    value.uneval(&mut file)
}

/// Convenience wrapper around [`to_file`] that resolves the path against the
/// build script's `OUT_DIR`.
pub fn to_out_dir<T: Uneval + ?Sized>(value: &T, file_name: impl AsRef<str>) -> io::Result<()> {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR not set, check if you're running this from the build script");
    let mut path = PathBuf::from(out_dir);
    path.push(file_name.as_ref());
    to_file(value, path)
}

/// Render the value to a `String` containing the generated Rust code.
///
/// The output is always valid UTF-8 because every blanket impl in this crate
/// writes through helpers that produce valid UTF-8 byte sequences, so this
/// function never fails for that reason.
pub fn to_string<T: Uneval + ?Sized>(value: &T) -> io::Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    value.uneval(&mut buf)?;
    Ok(String::from_utf8(buf).expect("uneval output is always valid UTF-8"))
}
