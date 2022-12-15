use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! (
    pub static ref BUILTIN_LIBS : HashMap<String, String> = HashMap::from([
        ("$std".to_owned(), include_str!("stdlib/std.rhl").to_owned())
    ]);
);