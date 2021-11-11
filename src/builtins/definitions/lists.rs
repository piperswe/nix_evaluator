use std::rc::Rc;

use rpds::Vector;

use crate::{
    builtins::{mismatch, BuiltinError, Result},
    value::Value,
};

pub fn concat_lists(lists: Value) -> Result {
    if let Value::List(lists) = lists {
        let mut res = Vector::new();
        for list in lists.iter() {
            if let Value::List(list) = list {
                for x in list.iter() {
                    res.push_back_mut(x.to_owned());
                }
            } else {
                return mismatch("list", list.to_owned());
            }
        }
        Ok(Value::List(res))
    } else {
        mismatch("list", lists)
    }
}

pub fn elem(x: Value) -> Result {
    Ok(Value::BuiltinFunction(Rc::new(move |xs| {
        if let Value::List(xs) = xs {
            Ok(xs.iter().any(|y| x.eq(y)).into())
        } else {
            mismatch("list", xs)
        }
    })))
}

pub fn elem_at(xs: Value) -> Result {
    if let Value::List(xs) = xs {
        Ok(Value::BuiltinFunction(Rc::new(move |n| {
            if let Value::Integer(n) = n {
                if n < 0 || (usize::BITS < (i64::BITS - 1) && n > (usize::MAX as i64)) {
                    Err(BuiltinError::OutOfBounds(n).into())
                } else if let Some(x) = xs.get(n as usize) {
                    Ok(x.to_owned())
                } else {
                    Err(BuiltinError::OutOfBounds(n).into())
                }
            } else {
                mismatch("integer", n)
            }
        })))
    } else {
        mismatch("list", xs)
    }
}

pub fn head(list: Value) -> Result {
    if let Value::List(list) = list {
        if let Some(x) = list.first() {
            Ok(x.to_owned())
        } else {
            Err(BuiltinError::OutOfBounds(0).into())
        }
    } else {
        mismatch("list", list)
    }
}

pub fn length(e: Value) -> Result {
    if let Value::List(e) = e {
        Ok((e.len() as i64).into())
    } else {
        mismatch("list", e)
    }
}

pub fn tail(list: Value) -> Result {
    if let Value::List(list) = list {
        let mut ret = Vector::new();
        for v in list.iter().skip(1) {
            ret.push_back_mut(v.to_owned());
        }
        Ok(Value::List(ret))
    } else {
        mismatch("list", list)
    }
}
