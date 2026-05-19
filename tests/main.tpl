use uneval::ser::{{Uneval, StringMode}};
use serde::Serialize;
use std::fs::File;
use batch_run::{{Batch, config::Config}};

mod definition;

fn main() {{
    let file = File::create("test_fixtures/{name}/generated.rs").unwrap();
    let mut ser = Uneval::new(file).string_mode(StringMode::{string_mode});
    ({value}).serialize(&mut ser).unwrap();
    drop(ser);
    let b = Batch::new();
    b.run_match("test_fixtures/{name}/{name}-user.rs");
    b.run_with_config(Config::from_env().unwrap().with_stderr_no_color()).unwrap().assert_all_ok();
}}