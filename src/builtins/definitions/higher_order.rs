use std::rc::Rc;

use rpds::{HashTrieMap, Vector};

use crate::{
    builtins::{mismatch, nyi, Result},
    value::Value,
};

pub fn all(pred: Value) -> Result {
    let pred = pred.materialize()?;
    if pred.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                for x in list.iter() {
                    let res = pred.clone().call(x.to_owned())?.materialize()?;
                    if let Value::Boolean(res) = res {
                        if !res {
                            return Ok(false.into());
                        }
                    } else {
                        return mismatch("boolean", res);
                    }
                }
                Ok(true.into())
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", pred)
    }
}

pub fn any(pred: Value) -> Result {
    let pred = pred.materialize()?;
    if pred.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                for x in list.iter() {
                    let res = pred.clone().call(x.to_owned())?.materialize()?;
                    if let Value::Boolean(res) = res {
                        if res {
                            return Ok(true.into());
                        }
                    } else {
                        return mismatch("boolean", res);
                    }
                }
                Ok(false.into())
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", pred)
    }
}

pub fn concat_map(f: Value) -> Result {
    let f = f.materialize()?;
    if f.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                let mut res = Vector::new();
                for x in list.iter() {
                    let f_res = f.clone().call(x.to_owned())?.materialize()?;
                    if let Value::List(f_res) = f_res {
                        for x in f_res.iter() {
                            res.push_back_mut(x.to_owned());
                        }
                    } else {
                        return mismatch("list", f_res);
                    }
                }
                Ok(Value::List(res))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", f)
    }
}

pub fn filter(f: Value) -> Result {
    let f = f.materialize()?;
    if f.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                let mut res = Vector::new();
                for x in list.iter() {
                    if let Value::Boolean(true) = f.clone().call(x.to_owned())?.materialize()? {
                        res.push_back_mut(x.to_owned());
                    }
                }
                Ok(Value::List(res))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", f)
    }
}

pub fn foldl(op: Value) -> Result {
    let op = op.materialize()?;
    if op.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |nul| {
            let op = op.clone();
            let nul = nul.materialize()?;
            Ok(Value::BuiltinFunction(Rc::new(move |list| {
                let list = list.materialize()?;
                if let Value::List(list) = list {
                    let mut accumulator = nul.clone();
                    for v in list.iter() {
                        accumulator = op
                            .clone()
                            .call(accumulator)?
                            .call(v.to_owned())?
                            .materialize()?;
                    }
                    Ok(accumulator)
                } else {
                    mismatch("list", list)
                }
            })))
        })))
    } else {
        mismatch("function", op)
    }
}

pub fn function_args(_: Value) -> Result {
    crate::evaluator::nyi("pattern matching")
}

pub fn gen_list(generator: Value) -> Result {
    let generator = generator.materialize()?;
    if generator.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |length| {
            let length = length.materialize()?;
            if let Value::Integer(length) = length {
                let mut v = Vector::new();
                for i in 0..length {
                    v.push_back_mut(generator.clone().call(i.into())?);
                }
                Ok(Value::List(v))
            } else {
                mismatch("integer", length)
            }
        })))
    } else {
        mismatch("function", generator)
    }
}

pub fn map(f: Value) -> Result {
    let f = f.materialize()?;
    if f.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                let mut res = Vector::new();
                for x in list.iter() {
                    res.push_back_mut(f.clone().call(x.to_owned())?);
                }
                Ok(Value::List(res))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", f)
    }
}

pub fn map_attrs(f: Value) -> Result {
    let f = f.materialize()?;
    if f.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |attrset| {
            let attrset = attrset.materialize()?;
            if let Value::AttrSet(attrset) = attrset {
                let mut res = HashTrieMap::new();
                for (k, v) in attrset.iter() {
                    res.insert_mut(
                        k.to_owned(),
                        f.clone().call(k.to_owned().into())?.call(v.to_owned())?,
                    );
                }
                Ok(Value::AttrSet(res))
            } else {
                mismatch("attribute set", f.clone())
            }
        })))
    } else {
        mismatch("function", f)
    }
}

pub fn partition(pred: Value) -> Result {
    let pred = pred.materialize()?;
    if pred.callable() {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                let mut right = Vector::new();
                let mut wrong = Vector::new();
                for x in list.iter() {
                    let res = pred.clone().call(x.to_owned())?;
                    if res == Value::Boolean(true) {
                        right.push_back_mut(x.to_owned());
                    } else if res == Value::Boolean(false) {
                        wrong.push_back_mut(x.to_owned());
                    } else {
                        return mismatch("boolean", res);
                    }
                }
                let mut res = HashTrieMap::new();
                res.insert_mut("right".into(), Value::List(right));
                res.insert_mut("wrong".into(), Value::List(wrong));
                Ok(Value::AttrSet(res))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("function", pred)
    }
}

pub fn sort(_: Value) -> Result {
    nyi("sort")
}
