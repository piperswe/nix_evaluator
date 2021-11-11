use std::rc::Rc;

use crate::{
    builtins::{mismatch, BuiltinError, Result},
    value::Value,
};

#[cfg(feature = "md5")]
fn hash_string_md5(s: Value) -> Result {
    let s = s.materialize()?;
    if let Value::String(s) = s {
        let digest = md5::compute(s.as_bytes());
        Ok(format!("{:x}", digest).into())
    } else {
        mismatch("string", s)
    }
}

#[cfg(not(feature = "md5"))]
fn hash_string_md5(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("md5".into()))
}

#[cfg(feature = "sha1")]
fn hash_string_sha1(s: Value) -> Result {
    use sha1::{Digest, Sha1};
    let s = s.materialize()?;
    if let Value::String(s) = s {
        let mut hasher = Sha1::new();
        hasher.update(s.as_bytes());
        let digest = hasher.finalize();
        Ok(format!("{:x}", digest).into())
    } else {
        mismatch("string", s)
    }
}

#[cfg(not(feature = "sha1"))]
fn hash_string_sha1(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("sha1".into()))
}

#[cfg(feature = "sha256")]
fn hash_string_sha256(s: Value) -> Result {
    use sha2::{Digest, Sha256};
    let s = s.materialize()?;
    if let Value::String(s) = s {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let digest = hasher.finalize();
        Ok(format!("{:x}", digest).into())
    } else {
        mismatch("string", s)
    }
}

#[cfg(not(feature = "sha256"))]
fn hash_string_sha256(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("sha256".into()))
}

#[cfg(feature = "sha512")]
fn hash_string_sha512(s: Value) -> Result {
    use sha2::{Digest, Sha512};
    let s = s.materialize()?;
    if let Value::String(s) = s {
        let mut hasher = Sha512::new();
        hasher.update(s.as_bytes());
        let digest = hasher.finalize();
        Ok(format!("{:x}", digest).into())
    } else {
        mismatch("string", s)
    }
}

#[cfg(not(feature = "sha512"))]
fn hash_string_sha512(_: Value) -> Result {
    use crate::evaluator::EvalError;

    Err(EvalError::NotEnabled("sha512".into()))
}

pub fn hash_string(t: Value) -> Result {
    let t = t.materialize()?;
    if let Value::String(t) = t {
        if t == "md5" {
            Ok(Value::BuiltinFunction(Rc::new(hash_string_md5)))
        } else if t == "sha1" {
            Ok(Value::BuiltinFunction(Rc::new(hash_string_sha1)))
        } else if t == "sha256" {
            Ok(Value::BuiltinFunction(Rc::new(hash_string_sha256)))
        } else if t == "sha512" {
            Ok(Value::BuiltinFunction(Rc::new(hash_string_sha512)))
        } else {
            Err(BuiltinError::UnknownHash(t.into()).into())
        }
    } else {
        mismatch("string", t)
    }
}

pub fn hash_file(_: Value) -> Result {
    crate::evaluator::nyi("derivations")
}
