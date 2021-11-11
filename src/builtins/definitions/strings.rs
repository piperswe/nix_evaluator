use std::{env, rc::Rc};

use rpds::HashTrieMap;

use crate::{
    builtins::{mismatch, nyi, BuiltinError, Result},
    value::Value,
};

#[cfg(feature = "compare_versions")]
pub fn compare_versions(s1: Value) -> Result {
    let s1 = s1.materialize()?;
    if let Value::String(s1) = s1 {
        Ok(Value::BuiltinFunction(Rc::new(move |s2| {
            let s2 = s2.materialize()?;
            if let Value::String(s2) = s2 {
                use version_compare::*;
                match compare(s1.clone(), s2.clone()) {
                    Ok(Cmp::Eq) => Ok(0.into()),
                    Ok(Cmp::Lt) => Ok((-1).into()),
                    Ok(Cmp::Gt) => Ok(1.into()),
                    Ok(_) => Err(BuiltinError::UnexpectedVersionOutput.into()),
                    Err(_) => Err(BuiltinError::VersionParse(s1.clone().into(), s2.into()).into()),
                }
            } else {
                mismatch("string", s2)
            }
        })))
    } else {
        mismatch("string", s1)
    }
}

#[cfg(not(feature = "compare_versions"))]
pub fn compare_versions(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("compare_versions".into()))
}

pub fn concat_strings_sep(separator: Value) -> Result {
    let separator = separator.materialize()?;
    if let Value::String(separator) = separator {
        Ok(Value::BuiltinFunction(Rc::new(move |list| {
            let list = list.materialize()?;
            if let Value::List(list) = list {
                Ok(list
                    .iter()
                    .map(|v| {
                        let v = v.to_owned().materialize()?;
                        if let Value::String(v) = v {
                            Ok(v)
                        } else {
                            mismatch("string", v.to_owned())
                        }
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join(&separator)
                    .into())
            } else {
                mismatch("list", list)
            }
        })))
    } else {
        mismatch("string", separator)
    }
}

#[cfg(feature = "json")]
pub fn from_json(e: Value) -> Result {
    let e = e.materialize()?;
    if let Value::String(e) = e {
        Ok(serde_json::from_str(&e).map_err(BuiltinError::from)?)
    } else {
        mismatch("string", e)
    }
}

#[cfg(not(feature = "json"))]
pub fn from_json(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("json".into()))
}

#[cfg(feature = "json")]
pub fn to_json(e: Value) -> Result {
    let e = e.materialize_deep()?;
    Ok(serde_json::to_string(&e)
        .map_err(BuiltinError::from)?
        .into())
}

#[cfg(not(feature = "json"))]
pub fn to_json(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("json".into()))
}

pub fn get_env(s: Value) -> Result {
    let s = s.materialize()?;
    if let Value::String(s) = s {
        match env::var(s.clone()) {
            Ok(x) => Ok(x.into()),
            Err(env::VarError::NotPresent) => Ok(Value::Null),
            Err(e) => Err(BuiltinError::Environment(s.into(), e).into()),
        }
    } else {
        mismatch("string", s)
    }
}

#[cfg(feature = "regex")]
pub fn f_match(regex: Value) -> Result {
    use regex::Regex;
    let regex = regex.materialize()?;
    if let Value::String(regex) = regex {
        let re = Regex::new(&regex).map_err(BuiltinError::from)?;
        Ok(Value::BuiltinFunction(Rc::new(move |str| {
            let str = str.materialize()?;
            if let Value::String(str) = str {
                if let Some(captures) = re.captures(&str) {
                    Ok(Value::List(
                        captures
                            .iter()
                            .filter_map(|x| x.map(|x| Value::String(x.as_str().to_string())))
                            .collect(),
                    ))
                } else {
                    Ok(Value::Null)
                }
            } else {
                mismatch("string", str)
            }
        })))
    } else {
        mismatch("string", regex)
    }
}

#[cfg(not(feature = "json"))]
pub fn f_match(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("regex".into()))
}

pub fn parse_drv_name(s: Value) -> Result {
    fn name_version_pair(name: String, version: Option<String>) -> Value {
        let mut attrset = HashTrieMap::new();
        attrset.insert_mut("name".into(), name.into());
        attrset.insert_mut("version".into(), version.map_or(Value::Null, Value::String));
        Value::AttrSet(attrset)
    }
    let s = s.materialize()?;
    if let Value::String(s) = s {
        Ok(s.clone().split_once('-').map_or_else(
            || name_version_pair(s, None),
            |(name, version)| name_version_pair(name.to_string(), Some(version.to_string())),
        ))
    } else {
        mismatch("string", s)
    }
}

pub fn replace_strings(from: Value) -> Result {
    let from = from.materialize_deep()?;
    if let Value::List(from) = from {
        Ok(Value::BuiltinFunction(Rc::new(move |to| {
            let to = to.materialize_deep()?;
            if let Value::List(to) = to {
                if from.len() == to.len() {
                    let l = from.len();
                    let from = from.clone();
                    Ok(Value::BuiltinFunction(Rc::new(move |s| {
                        let s = s.materialize()?;
                        if let Value::String(s) = s {
                            let mut res = s.clone();
                            for i in 0..l {
                                let from = if let Some(from) = from.get(i) {
                                    if let Value::String(from) = from {
                                        from
                                    } else {
                                        return mismatch("string", from.to_owned());
                                    }
                                } else {
                                    panic!("BUG: from.len != to.len");
                                };
                                let to = if let Some(to) = to.get(i) {
                                    if let Value::String(to) = to {
                                        to
                                    } else {
                                        return mismatch("string", to.to_owned());
                                    }
                                } else {
                                    panic!("BUG: from.len != to.len");
                                };
                                res = res.replace(from, to);
                            }
                            Ok(s.into())
                        } else {
                            mismatch("string", s)
                        }
                    })))
                } else {
                    Err(BuiltinError::ReplaceStringsArgLength.into())
                }
            } else {
                mismatch("list", to)
            }
        })))
    } else {
        mismatch("list", from)
    }
}

#[cfg(feature = "regex")]
pub fn split(regex: Value) -> Result {
    use regex::Regex;
    let regex = regex.materialize()?;
    if let Value::String(regex) = regex {
        let re = Regex::new(&regex).map_err(BuiltinError::from)?;
        Ok(Value::BuiltinFunction(Rc::new(move |str| {
            let str = str.materialize()?;
            if let Value::String(str) = str {
                Ok(Value::List(
                    re.split(&str).map(|x| x.to_string().into()).collect(),
                ))
            } else {
                mismatch("string", str)
            }
        })))
    } else {
        mismatch("string", regex)
    }
}

#[cfg(not(feature = "json"))]
pub fn split(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("regex".into()))
}

pub fn split_version(_: Value) -> Result {
    nyi("splitVersion")
}

pub fn string_length(e: Value) -> Result {
    let e = e.materialize()?;
    if let Value::String(e) = e {
        Ok(Value::Integer(e.len() as i64))
    } else {
        mismatch("string", e)
    }
}

pub fn substring(start: Value) -> Result {
    let start = start.materialize()?;
    if let Value::Integer(start) = start {
        Ok(Value::BuiltinFunction(Rc::new(move |len| {
            if let Value::Integer(len) = len {
                Ok(Value::BuiltinFunction(Rc::new(move |s| {
                    if let Value::String(s) = s {
                        Ok(s[(start as usize)..((start + len) as usize)]
                            .to_string()
                            .into())
                    } else {
                        mismatch("string", s)
                    }
                })))
            } else {
                mismatch("integer", len)
            }
        })))
    } else {
        mismatch("integer", start)
    }
}

pub fn to_string(e: Value) -> Result {
    match &e {
        Value::String(_) => Ok(e),
        Value::Integer(x) => Ok(format!("{}", *x).into()),
        Value::Floating(x) => Ok(format!("{}", *x).into()),
        Value::Path(_) => todo!(),
        Value::Boolean(x) => {
            if *x {
                Ok("1".to_string().into())
            } else {
                Ok("".to_string().into())
            }
        }
        Value::Null => Ok("".to_string().into()),
        Value::Function(_, _, _) => Err(BuiltinError::CannotSerialize("function".into()).into()),
        Value::AttrSet(set) => {
            if let Some(to_string) = set.get("__toString") {
                if to_string.callable() {
                    let res = to_string.clone().call(e)?;
                    if let Value::String(_) = res {
                        Ok(res)
                    } else {
                        mismatch("string", res)
                    }
                } else {
                    mismatch("function", to_string.to_owned())
                }
            } else if let Some(out_path) = set.get("outPath") {
                if let Value::String(_) = out_path {
                    Ok(out_path.to_owned())
                } else {
                    mismatch("string", out_path.to_owned())
                }
            } else {
                Err(BuiltinError::CannotSerialize("attribute set".into()).into())
            }
        }
        Value::List(l) => {
            let mut ret = String::new();
            for v in l.iter() {
                let str = to_string(v.to_owned())?;
                if let Value::String(str) = str {
                    ret += &str;
                    ret += " ";
                } else {
                    panic!(
                        "BUG: toString returned something other than a string: {}",
                        str
                    );
                }
            }
            ret.remove(ret.len() - 1);
            Ok(ret.into())
        }
        Value::Thunk(_, _) => to_string(e.materialize()?),
        Value::BuiltinFunction(_) => Err(BuiltinError::CannotSerialize("function".into()).into()),
    }
}

pub fn to_xml(_: Value) -> Result {
    nyi("toXML")
}
