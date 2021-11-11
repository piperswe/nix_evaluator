use std::rc::Rc;

use crate::{
    builtins::{nyi, BuiltinError, Result},
    value::Value,
};

pub fn abort(s: Value) -> Result {
    Err(BuiltinError::Aborted(s.to_string()).into())
}

pub fn throw(s: Value) -> Result {
    Err(BuiltinError::Thrown(s.to_string()).into())
}

pub fn trace(e1: Value) -> Result {
    eprintln!("trace: {:?}", e1.materialize());
    Ok(Value::BuiltinFunction(Rc::new(Ok)))
}

pub fn try_eval(_: Value) -> Result {
    nyi("tryEval")
}
