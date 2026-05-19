mod definition;
use definition::{{{types}}};

const ITEM: {ser_type} = include!("generated.rs");

fn main() {{
    assert_eq!(ITEM, {value});
}}