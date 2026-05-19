//! Blanket `Uneval` implementations for primitives, standard library types,
//! and structural wrappers (references, smart pointers, collections, tuples).
//!
//! Each impl picks the constructor expression that suits its own type: bare
//! literals for `&str`, `"…".to_string()` for `String`, `vec![…]` for `Vec<T>`,
//! `Cow::Borrowed(…)` for `Cow<'static, str>`, and so on. This is what lets the
//! derive macro emit `self.field.uneval(w)?` for every field without knowing
//! which constructor expression that field's type prefers.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::io::{self, Write};

use crate::Uneval;
use crate::helpers::{write_char_literal, write_str_literal};

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Uneval for $t {
                fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
                    write!(w, "{}{}", self, stringify!($t))
                }
            }
        )*
    };
}

impl_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl Uneval for $t {
                fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
                    if self.is_nan() {
                        write!(w, "{}::NAN", stringify!($t))
                    } else if self.is_infinite() {
                        if *self > 0.0 {
                            write!(w, "{}::INFINITY", stringify!($t))
                        } else {
                            write!(w, "{}::NEG_INFINITY", stringify!($t))
                        }
                    } else {
                        write!(w, "{:?}{}", self, stringify!($t))
                    }
                }
            }
        )*
    };
}

impl_float!(f32, f64);

impl Uneval for bool {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write!(w, "{}", self)
    }
}

impl Uneval for char {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_char_literal(w, *self)
    }
}

impl Uneval for str {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_str_literal(w, self)
    }
}

impl Uneval for String {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_str_literal(w, self)?;
        w.write_all(b".to_string()")
    }
}

impl Uneval for Cow<'_, str> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(b"::std::borrow::Cow::Borrowed(")?;
        write_str_literal(w, self.as_ref())?;
        w.write_all(b")")
    }
}

impl<T: ?Sized + Uneval> Uneval for &T {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        (**self).uneval(w)
    }
}

impl<T: ?Sized + Uneval> Uneval for &mut T {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        (**self).uneval(w)
    }
}

impl<T: ?Sized + Uneval> Uneval for Box<T> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(b"::std::boxed::Box::new(")?;
        (**self).uneval(w)?;
        w.write_all(b")")
    }
}

impl<T: Uneval> Uneval for Option<T> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            None => w.write_all(b"::std::option::Option::None"),
            Some(v) => {
                w.write_all(b"::std::option::Option::Some(")?;
                v.uneval(w)?;
                w.write_all(b")")
            }
        }
    }
}

impl<T: Uneval, E: Uneval> Uneval for Result<T, E> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Ok(v) => {
                w.write_all(b"::std::result::Result::Ok(")?;
                v.uneval(w)?;
                w.write_all(b")")
            }
            Err(e) => {
                w.write_all(b"::std::result::Result::Err(")?;
                e.uneval(w)?;
                w.write_all(b")")
            }
        }
    }
}

impl Uneval for () {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(b"()")
    }
}

macro_rules! impl_tuple {
    ($($t:ident),+) => {
        impl<$($t: Uneval),+> Uneval for ($($t,)+) {
            fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
                #[allow(non_snake_case)]
                let ($($t,)+) = self;
                w.write_all(b"(")?;
                let mut __first = true;
                $(
                    if !__first { w.write_all(b", ")?; }
                    __first = false;
                    $t.uneval(w)?;
                )+
                w.write_all(b",)")
            }
        }
    };
}

impl_tuple!(T0);
impl_tuple!(T0, T1);
impl_tuple!(T0, T1, T2);
impl_tuple!(T0, T1, T2, T3);
impl_tuple!(T0, T1, T2, T3, T4);
impl_tuple!(T0, T1, T2, T3, T4, T5);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

fn write_iter<'a, I, T>(w: &mut dyn Write, open: &[u8], close: &[u8], items: I) -> io::Result<()>
where
    I: IntoIterator<Item = &'a T>,
    T: 'a + Uneval,
{
    w.write_all(open)?;
    let mut first = true;
    for item in items {
        if !first {
            w.write_all(b", ")?;
        }
        first = false;
        item.uneval(w)?;
    }
    w.write_all(close)
}

impl<T: Uneval, const N: usize> Uneval for [T; N] {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_iter(w, b"[", b"]", self.iter())
    }
}

impl<T: Uneval> Uneval for [T] {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_iter(w, b"[", b"]", self.iter())
    }
}

impl<T: Uneval> Uneval for Vec<T> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_iter(w, b"::std::vec![", b"]", self.iter())
    }
}

impl<T: Uneval> Uneval for VecDeque<T> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(b"::std::collections::VecDeque::from(")?;
        write_iter(w, b"::std::vec![", b"]", self.iter())?;
        w.write_all(b")")
    }
}

fn write_map<'a, I, K, V>(w: &mut dyn Write, ty: &[u8], entries: I) -> io::Result<()>
where
    I: IntoIterator<Item = (&'a K, &'a V)>,
    K: 'a + Uneval,
    V: 'a + Uneval,
{
    w.write_all(ty)?;
    w.write_all(b"::from([")?;
    let mut first = true;
    for (k, v) in entries {
        if !first {
            w.write_all(b", ")?;
        }
        first = false;
        w.write_all(b"(")?;
        k.uneval(w)?;
        w.write_all(b", ")?;
        v.uneval(w)?;
        w.write_all(b")")?;
    }
    w.write_all(b"])")
}

impl<K: Uneval, V: Uneval, S> Uneval for HashMap<K, V, S> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_map(w, b"::std::collections::HashMap", self.iter())
    }
}

impl<K: Uneval, V: Uneval> Uneval for BTreeMap<K, V> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_map(w, b"::std::collections::BTreeMap", self.iter())
    }
}

fn write_set<'a, I, T>(w: &mut dyn Write, ty: &[u8], items: I) -> io::Result<()>
where
    I: IntoIterator<Item = &'a T>,
    T: 'a + Uneval,
{
    w.write_all(ty)?;
    write_iter(w, b"::from([", b"])", items)
}

impl<T: Uneval, S> Uneval for HashSet<T, S> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_set(w, b"::std::collections::HashSet", self.iter())
    }
}

impl<T: Uneval> Uneval for BTreeSet<T> {
    fn uneval(&self, w: &mut dyn Write) -> io::Result<()> {
        write_set(w, b"::std::collections::BTreeSet", self.iter())
    }
}
