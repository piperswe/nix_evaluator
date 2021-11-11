use crate::{builtins::Result, value::Value};

const T: Value = Value::Boolean(true);
const F: Value = Value::Boolean(false);

pub fn is_attrs(e: Value) -> Result {
    if let Value::AttrSet(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_bool(e: Value) -> Result {
    if let Value::Boolean(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_float(e: Value) -> Result {
    if let Value::Floating(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_function(e: Value) -> Result {
    let e = e.materialize()?;
    if let Value::Function(_, _, _) = e {
        Ok(T)
    } else if let Value::BuiltinFunction(_) = e {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_int(e: Value) -> Result {
    if let Value::Integer(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_list(e: Value) -> Result {
    if let Value::List(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_null(e: Value) -> Result {
    Ok((e.materialize()? == Value::Null).into())
}

pub fn is_path(e: Value) -> Result {
    if let Value::Path(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn is_string(e: Value) -> Result {
    if let Value::String(_) = e.materialize()? {
        Ok(T)
    } else {
        Ok(F)
    }
}

pub fn type_of(e: Value) -> Result {
    fn s(s: &'static str) -> Result {
        Ok(Value::String(s.to_string()))
    }
    match e {
        Value::String(_) => s("string"),
        Value::Integer(_) => s("int"),
        Value::Floating(_) => s("float"),
        Value::Path(_) => s("path"),
        Value::Boolean(_) => s("bool"),
        Value::Null => s("null"),
        Value::Function(_, _, _) => s("lambda"),
        Value::AttrSet(_) => s("set"),
        Value::List(_) => s("list"),
        Value::Thunk(_, _) => type_of(e.materialize()?),
        Value::BuiltinFunction(_) => s("lambda"),
    }
}
