use std::{
    cmp::Ordering,
    fmt::{self, Display},
    rc::Rc,
};

use rnix::SyntaxNode;
use rpds::{HashTrieMap, Vector};
use thiserror::Error;

use crate::{
    evaluator::{eval_ctx, EvalError, EvaluationContext},
    ErrorString,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericValue {
    Integer(i64),
    Floating(f64),
}

#[derive(Clone)]
pub enum Value {
    // Scalar types
    String(String),
    Integer(i64),
    Floating(f64),
    Path(String),
    Boolean(bool),
    Null,

    // Complex types
    Function(String, EvaluationContext, SyntaxNode),
    AttrSet(HashTrieMap<String, Value>),
    List(Vector<Value>),

    // Special types
    Thunk(EvaluationContext, SyntaxNode),
    BuiltinFunction(Rc<dyn Fn(Value) -> Result<Value, EvalError>>),
}

impl From<String> for Value {
    fn from(x: String) -> Self {
        Value::String(x)
    }
}

impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Value::Integer(x)
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Self {
        Value::Floating(x)
    }
}

impl From<bool> for Value {
    fn from(x: bool) -> Self {
        Value::Boolean(x)
    }
}

#[derive(Error, Debug)]
pub enum ArithmeticError {
    #[error("Type mismatch - expected {0}, found {1}")]
    TypeMismatch(ErrorString, ErrorString),
    #[error("Overflow/underflow occurred performing arithmetic on {0:?} and {1:?}")]
    Overflow(NumericValue, NumericValue),
    #[error("Divide by zero")]
    DivideByZero,
    #[error("Comparison between {0:?} and {1:?} is impossible")]
    ImpossibleComparison(NumericValue, NumericValue),
}

enum Normalized {
    Integer(i64, i64),
    Floating(f64, f64),
}

fn normalize_numerics(a: &Value, b: &Value) -> Result<Normalized, ArithmeticError> {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Normalized::Integer(*a, *b)),
        (Value::Floating(a), Value::Floating(b)) => Ok(Normalized::Floating(*a, *b)),
        (Value::Integer(a), Value::Floating(b)) => Ok(Normalized::Floating(*a as f64, *b)),
        (Value::Floating(a), Value::Integer(b)) => Ok(Normalized::Floating(*a, *b as f64)),
        _ => Err(ArithmeticError::TypeMismatch(
            "numbers".into(),
            format!(
                "{} and {}",
                a.human_readable_type(),
                b.human_readable_type()
            )
            .into(),
        )),
    }
}

impl Value {
    pub fn human_readable_type(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Floating(_) => "floating-point number",
            Value::Path(_) => "path",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::Function(_, _, _) => "function",
            Value::AttrSet(_) => "attribute set",
            Value::List(_) => "list",
            Value::Thunk(_, _) => "thunk",
            Value::BuiltinFunction(_) => "built-in function",
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Floating(_))
    }

    pub fn add(&self, rhs_v: &Value) -> Result<Value, ArithmeticError> {
        match normalize_numerics(self, rhs_v)? {
            Normalized::Integer(lhs, rhs) => Ok(lhs
                .checked_add(rhs)
                .ok_or_else(|| ArithmeticError::Overflow(self.into(), rhs_v.into()))?
                .into()),
            Normalized::Floating(lhs, rhs) => Ok((lhs + rhs).into()),
        }
    }

    pub fn sub(&self, rhs_v: &Value) -> Result<Value, ArithmeticError> {
        match normalize_numerics(self, rhs_v)? {
            Normalized::Integer(lhs, rhs) => Ok(lhs
                .checked_sub(rhs)
                .ok_or_else(|| ArithmeticError::Overflow(self.into(), rhs_v.into()))?
                .into()),
            Normalized::Floating(lhs, rhs) => Ok((lhs - rhs).into()),
        }
    }

    pub fn mul(&self, rhs_v: &Value) -> Result<Value, ArithmeticError> {
        match normalize_numerics(self, rhs_v)? {
            Normalized::Integer(lhs, rhs) => Ok(lhs
                .checked_mul(rhs)
                .ok_or_else(|| ArithmeticError::Overflow(self.into(), rhs_v.into()))?
                .into()),
            Normalized::Floating(lhs, rhs) => Ok((lhs * rhs).into()),
        }
    }

    pub fn div(&self, rhs_v: &Value) -> Result<Value, ArithmeticError> {
        match normalize_numerics(self, rhs_v)? {
            Normalized::Integer(lhs, rhs) => {
                if rhs == 0 {
                    Err(ArithmeticError::DivideByZero)
                } else {
                    Ok(lhs
                        .checked_div(rhs)
                        .ok_or_else(|| ArithmeticError::Overflow(self.into(), rhs_v.into()))?
                        .into())
                }
            }
            Normalized::Floating(lhs, rhs) => Ok((lhs / rhs).into()),
        }
    }

    pub fn compare(&self, rhs_v: &Value) -> Result<Ordering, ArithmeticError> {
        Ok(match normalize_numerics(self, rhs_v)? {
            Normalized::Integer(lhs, rhs) => lhs.cmp(&rhs),
            Normalized::Floating(lhs, rhs) => lhs
                .partial_cmp(&rhs)
                .ok_or_else(|| ArithmeticError::ImpossibleComparison(self.into(), rhs_v.into()))?,
        })
    }

    pub fn materializable(&self) -> bool {
        match self {
            Self::Thunk(_, _) => true,
            Self::AttrSet(set) => set.values().any(|x| x.materializable()),
            Self::List(list) => list.iter().any(|x| x.materializable()),
            _ => false,
        }
    }

    pub fn materialize(self) -> Result<Self, EvalError> {
        if let Self::Thunk(ctx, body) = self {
            eval_ctx(body, ctx)
        } else {
            Ok(self)
        }
    }

    pub fn materialize_deep(self) -> Result<Self, EvalError> {
        let materialized = self.materialize()?;
        if let Self::AttrSet(set) = materialized {
            let mut deeply_materialized = set.clone();
            for k in set.keys() {
                if let Some(v) = set.get(k) {
                    if v.materializable() {
                        deeply_materialized
                            .insert_mut(k.to_string(), v.to_owned().materialize_deep()?);
                    }
                }
            }
            Ok(Self::AttrSet(deeply_materialized))
        } else if let Self::List(list) = materialized {
            let mut deeply_materialized = list.clone();
            for k in 0..list.len() {
                if let Some(v) = list.get(k) {
                    if v.materializable() {
                        deeply_materialized.set_mut(k, v.to_owned().materialize_deep()?);
                    }
                }
            }
            Ok(Self::List(deeply_materialized))
        } else {
            Ok(materialized)
        }
    }

    pub fn callable(&self) -> bool {
        matches!(self, Value::Function(_, _, _) | Value::BuiltinFunction(_))
    }

    pub fn call(self, val: Value) -> Result<Self, EvalError> {
        if let Self::Function(param, ctx, body) = self {
            let ctx = ctx.with(param, val);
            eval_ctx(body, ctx)
        } else if let Self::BuiltinFunction(f) = self {
            f(val)
        } else {
            Err(EvalError::TypeMismatch(
                "function".into(),
                self.human_readable_type().into(),
            ))
        }
    }

    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent_count: usize) -> fmt::Result {
        match self {
            Value::String(x) => write!(f, "\"{}\"", x),
            Value::Integer(x) => write!(f, "{}", x),
            Value::Floating(x) => write!(f, "{}", x),
            Value::Path(x) => write!(f, "{}", x),
            Value::Boolean(x) => write!(f, "{}", x),
            Value::Null => write!(f, "null"),
            Value::AttrSet(x) => {
                let indent = " ".repeat(indent_count);
                let body_ident = " ".repeat(indent_count + 2);
                writeln!(f, "{{")?;
                for (k, v) in x.iter() {
                    write!(f, "{}{} = ", body_ident, k)?;
                    v.fmt_indented(f, indent_count + 2)?;
                    writeln!(f)?;
                }
                write!(f, "{}}}", indent)
            }
            Value::List(x) => {
                let indent = " ".repeat(indent_count);
                let body_ident = " ".repeat(indent_count + 2);
                writeln!(f, "[")?;
                for v in x.iter() {
                    write!(f, "{}", body_ident)?;
                    v.fmt_indented(f, indent_count + 2)?;
                    writeln!(f)?;
                }
                write!(f, "{}]", indent)
            }
            Value::Thunk(ctx, node) => eval_ctx(node.to_owned(), ctx.to_owned())
                .map_err(|_| fmt::Error)?
                .fmt_indented(f, indent_count),
            _ => write!(f, "<{}>", self.human_readable_type()),
        }
    }
}

impl From<&Value> for NumericValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(x) => NumericValue::Integer(*x),
            Value::Floating(x) => NumericValue::Floating(*x),
            _ => panic!(
                "Only transform a Value into a NumericValue if you know the value is numeric!"
            ),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 2)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Floating(l0), Self::Floating(r0)) => l0 == r0,
            (Self::Path(l0), Self::Path(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Function(l0, l1, l2), Self::Function(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::AttrSet(l0), Self::AttrSet(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Thunk(l0, l1), Self::Thunk(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::BuiltinFunction(_), Self::BuiltinFunction(_)) => false,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
