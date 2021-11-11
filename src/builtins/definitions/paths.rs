use std::path::Path;

use crate::{
    builtins::{mismatch, nyi, Result},
    value::Value,
};

pub fn base_name_of(_: Value) -> Result {
    nyi("baseNameOf")
}

pub fn dir_of(s: Value) -> Result {
    let s = s.materialize()?;
    if let Value::String(s) = s {
        let path = Path::new(&s);
        Ok(path
            .parent()
            .map_or(s.clone().into(), |x| x.to_string_lossy().to_string().into()))
    } else {
        mismatch("string", s)
    }
}
