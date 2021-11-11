use std::rc::Rc;

use rpds::{HashTrieMap, Vector};

use crate::{
    builtins::{mismatch, BuiltinError, Result},
    value::Value,
};

pub fn attr_names(set: Value) -> Result {
    if let Value::AttrSet(set) = set {
        Ok(Value::List(
            set.keys().map(|k| Value::String(k.to_owned())).collect(),
        ))
    } else {
        mismatch("attribute set", set)
    }
}

pub fn attr_values(set: Value) -> Result {
    if let Value::AttrSet(set) = set {
        Ok(Value::List(set.values().map(|v| v.to_owned()).collect()))
    } else {
        mismatch("attribute set", set)
    }
}

pub fn cat_attrs(attr: Value) -> Result {
    if let Value::String(attr) = attr {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            if let Value::List(list) = list {
                let mut result = Vector::new();
                for x in list.iter() {
                    if let Value::AttrSet(set) = x {
                        if let Some(v) = set.get(&attr) {
                            result.push_back_mut(v.to_owned());
                        }
                    } else {
                        return mismatch("attribute set", x.to_owned());
                    }
                }
                Ok(Value::List(result))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("string", attr)
    }
}

pub fn get_attr(s: Value) -> Result {
    if let Value::String(s) = s {
        Ok(Value::BuiltinFunction(Rc::new(move |set| {
            if let Value::AttrSet(set) = set {
                Ok(set.get(&s).map_or(Value::Null, ToOwned::to_owned))
            } else {
                mismatch("attribute set", set)
            }
        })))
    } else {
        mismatch("string", s)
    }
}

pub fn has_attr(s: Value) -> Result {
    if let Value::String(s) = s {
        Ok(Value::BuiltinFunction(Rc::new(move |set| {
            if let Value::AttrSet(set) = set {
                Ok(set.contains_key(&s).into())
            } else {
                mismatch("attribute set", set)
            }
        })))
    } else {
        mismatch("string", s)
    }
}

pub fn intersect_attrs(e1: Value) -> Result {
    if let Value::AttrSet(e1) = e1 {
        Ok(Value::BuiltinFunction(Rc::new(move |e2| {
            if let Value::AttrSet(e2) = e2 {
                let mut res = e2.clone();
                for (k, _) in e2.iter() {
                    if !e1.contains_key(k) {
                        res.remove_mut(k);
                    }
                }
                Ok(Value::AttrSet(res))
            } else {
                mismatch("attribute set", e2)
            }
        })))
    } else {
        mismatch("attribute set", e1)
    }
}

pub fn list_to_attrs(e: Value) -> Result {
    if let Value::List(e) = e {
        let mut attrs = HashTrieMap::new();
        for v in e.iter() {
            if let Value::AttrSet(v) = v {
                if let Some(name) = v.get("name") {
                    if let Value::String(name) = name {
                        if let Some(value) = v.get("value") {
                            attrs.insert_mut(name.to_string(), value.to_owned());
                        } else {
                            return Err(BuiltinError::MissingAttr("value".into()).into());
                        }
                    } else {
                        return mismatch("string", name.to_owned());
                    }
                } else {
                    return Err(BuiltinError::MissingAttr("name".into()).into());
                }
            } else {
                return mismatch("attribute set", v.to_owned());
            }
        }
        Ok(Value::AttrSet(attrs))
    } else {
        mismatch("list", e)
    }
}

pub fn remove_attrs(set: Value) -> Result {
    if let Value::AttrSet(set) = set {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            if let Value::List(list) = list {
                let mut new_set = set.clone();
                for remove in list.iter() {
                    if let Value::String(remove) = remove {
                        if new_set.contains_key(remove) {
                            new_set.remove_mut(remove);
                        }
                    } else {
                        return mismatch("string", remove.to_owned());
                    }
                }
                Ok(Value::AttrSet(new_set))
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("attribute set", set)
    }
}
