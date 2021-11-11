use std::rc::Rc;

use crate::{
    builtins::{mismatch, Result},
    value::Value,
};

fn binary_numeric<F: 'static + Fn(Value, Value) -> Result>(e1: Value, f: F) -> Result {
    let e1 = e1.materialize()?;
    if e1.is_numeric() {
        Ok(Value::BuiltinFunction(Rc::new(move |e2| {
            let e2 = e2.materialize()?;
            if e2.is_numeric() {
                f(e1.clone(), e2)
            } else {
                mismatch("numeric", e2)
            }
        })))
    } else {
        mismatch("numeric", e1)
    }
}

pub fn add(e1: Value) -> Result {
    binary_numeric(e1, |a, b| Ok(a.add(&b)?))
}

pub fn sub(e1: Value) -> Result {
    binary_numeric(e1, |a, b| Ok(a.sub(&b)?))
}

pub fn mul(e1: Value) -> Result {
    binary_numeric(e1, |a, b| Ok(a.mul(&b)?))
}

pub fn div(e1: Value) -> Result {
    binary_numeric(e1, |a, b| Ok(a.div(&b)?))
}

fn binary_integral<F: 'static + Fn(i64, i64) -> Result>(e1: Value, f: F) -> Result {
    let e1 = e1.materialize()?;
    if let Value::Integer(e1) = e1 {
        Ok(Value::BuiltinFunction(Rc::new(move |e2| {
            let e2 = e2.materialize()?;
            if let Value::Integer(e2) = e2 {
                f(e1, e2)
            } else {
                mismatch("integer", e2)
            }
        })))
    } else {
        mismatch("integer", e1)
    }
}

pub fn bit_and(e1: Value) -> Result {
    binary_integral(e1, |a, b| Ok((a & b).into()))
}

pub fn bit_or(e1: Value) -> Result {
    binary_integral(e1, |a, b| Ok((a | b).into()))
}

pub fn bit_xor(e1: Value) -> Result {
    binary_integral(e1, |a, b| Ok((a ^ b).into()))
}

pub fn ceil(double: Value) -> Result {
    let double = double.materialize()?;
    if let Value::Floating(double) = double {
        Ok(double.ceil().into())
    } else {
        mismatch("floating-point number", double)
    }
}

pub fn floor(double: Value) -> Result {
    let double = double.materialize()?;
    if let Value::Floating(double) = double {
        Ok(double.floor().into())
    } else {
        mismatch("floating-point number", double)
    }
}

pub fn less_than(e1: Value) -> Result {
    binary_numeric(e1, |a, b| Ok(a.compare(&b)?.is_lt().into()))
}
