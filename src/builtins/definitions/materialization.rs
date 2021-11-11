use std::rc::Rc;

use crate::{builtins::Result, value::Value};

fn identity(x: Value) -> Result {
    Ok(x)
}

pub fn seq(s1: Value) -> Result {
    s1.materialize()?;
    Ok(Value::BuiltinFunction(Rc::new(identity)))
}

pub fn deep_seq(s1: Value) -> Result {
    s1.materialize_deep()?;
    Ok(Value::BuiltinFunction(Rc::new(identity)))
}
