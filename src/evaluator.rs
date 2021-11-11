use std::fmt;

use rnix::{types::*, value::ValueError, NixValue, StrPart, SyntaxNode};
use rpds::{HashTrieMap, List, Vector};
use thiserror::Error;

use crate::{
    builtins::{base_context, BuiltinError},
    value::{ArithmeticError, Value},
    ErrorString,
};

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("BUG: Node type mismatch")]
    Mismatch,
    #[error("BUG: Node missing required children")]
    MissingChildren,
    #[error("An error occurred parsing a literal")]
    LiteralParse(#[from] ValueError),
    #[error("Unexpected token")]
    UnexpectedToken,
    #[error("Unexpected node")]
    UnexpectedNode,
    #[error("Type mismatch - expected {0}, found {1}")]
    TypeMismatch(ErrorString, ErrorString),
    #[error("Feature not yet implemented: {0}")]
    NotYetImplemented(ErrorString),
    #[error("Feature flag {0} not enabled")]
    NotEnabled(ErrorString),
    #[error("Unresolved identifier {0}")]
    UnresolvedIdent(ErrorString),
    #[error("No such index {0} in attrset")]
    NoSuchIndex(ErrorString),

    #[error(transparent)]
    Arithmetic(#[from] ArithmeticError),
    #[error(transparent)]
    Builtin(#[from] BuiltinError),
}

type Result<T> = std::result::Result<T, EvalError>;

#[derive(Clone, PartialEq)]
pub struct EvaluationContext(HashTrieMap<String, Value>);

fn assoc_in(
    set: HashTrieMap<String, Value>,
    path: List<String>,
    val: Value,
) -> HashTrieMap<String, Value> {
    if path.is_empty() {
        panic!("BUG: path should never be less than 1 len!")
    } else if path.len() == 1 {
        set.insert(path.first().unwrap().to_owned(), val)
    } else if let Some(child) = set.get(path.first().unwrap()) {
        if let Value::AttrSet(child) = child {
            set.insert(
                path.first().unwrap().to_owned(),
                Value::AttrSet(assoc_in(child.clone(), path.drop_first().unwrap(), val)),
            )
        } else {
            panic!()
        }
    } else {
        set.insert(
            path.first().unwrap().to_owned(),
            Value::AttrSet(assoc_in(
                HashTrieMap::new(),
                path.drop_first().unwrap(),
                val,
            )),
        )
    }
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self(base_context())
    }

    pub fn with(&self, ident: String, val: Value) -> Self {
        Self(self.0.insert(ident, val))
    }

    pub fn with_path(&self, path: &[String], val: Value) -> Self {
        Self(assoc_in(
            self.0.clone(),
            path.iter().map(ToOwned::to_owned).collect(),
            val,
        ))
    }

    pub fn get(&self, ident: &str) -> Option<&Value> {
        self.0.get(ident)
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EvaluationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<evaluation context>")
    }
}

pub(crate) fn nyi<S: Into<ErrorString>, T>(feature: S) -> Result<T> {
    Err(EvalError::NotYetImplemented(feature.into()))
}

fn cast<T: TypedNode>(from: SyntaxNode) -> Result<T> {
    T::cast(from).ok_or(EvalError::Mismatch)
}

fn expect_child<T>(x: Option<T>) -> Result<T> {
    x.ok_or(EvalError::MissingChildren)
}

fn eval_root(node: Root, context: EvaluationContext) -> Result<Value> {
    eval_ctx(expect_child(node.inner())?, context)
}

fn eval_literal(node: rnix::types::Value) -> Result<Value> {
    Ok(match node.to_value()? {
        NixValue::Float(x) => Value::Floating(x),
        NixValue::Integer(x) => Value::Integer(x),
        NixValue::String(x) => Value::String(x),
        NixValue::Path(_, x) => Value::Path(x),
    })
}

fn eval_bin_op(node: BinOp, context: EvaluationContext) -> Result<Value> {
    let lhs = eval_ctx(expect_child(node.lhs())?, context.clone())?;
    let rhs_node = expect_child(node.rhs())?;
    let rhs = || eval_ctx(rhs_node, context);
    match node.operator() {
        BinOpKind::Concat => nyi("concatenation"),
        BinOpKind::IsSet => nyi("isset"),
        BinOpKind::Update => nyi("updating"),
        BinOpKind::Add => Ok(lhs.add(&rhs()?)?),
        BinOpKind::Sub => Ok(lhs.sub(&rhs()?)?),
        BinOpKind::Mul => Ok(lhs.mul(&rhs()?)?),
        BinOpKind::Div => Ok(lhs.div(&rhs()?)?),
        BinOpKind::And => {
            if let Value::Boolean(lhs) = lhs {
                if lhs {
                    rhs()
                } else {
                    Ok(false.into())
                }
            } else {
                Err(EvalError::TypeMismatch(
                    "boolean".into(),
                    lhs.human_readable_type().into(),
                ))
            }
        }
        BinOpKind::Or => {
            if let Value::Boolean(lhs) = lhs {
                if lhs {
                    Ok(true.into())
                } else {
                    rhs()
                }
            } else {
                Err(EvalError::TypeMismatch(
                    "boolean".into(),
                    lhs.human_readable_type().into(),
                ))
            }
        }
        BinOpKind::Implication => {
            if let Value::Boolean(lhs) = lhs {
                if lhs {
                    rhs()
                } else {
                    Ok(true.into())
                }
            } else {
                Err(EvalError::TypeMismatch(
                    "boolean".into(),
                    lhs.human_readable_type().into(),
                ))
            }
        }
        BinOpKind::Equal => Ok(lhs.compare(&rhs()?)?.is_eq().into()),
        BinOpKind::Less => Ok(lhs.compare(&rhs()?)?.is_lt().into()),
        BinOpKind::LessOrEq => Ok(lhs.compare(&rhs()?)?.is_le().into()),
        BinOpKind::More => Ok(lhs.compare(&rhs()?)?.is_gt().into()),
        BinOpKind::MoreOrEq => Ok(lhs.compare(&rhs()?)?.is_ge().into()),
        BinOpKind::NotEqual => Ok(lhs.compare(&rhs()?)?.is_ne().into()),
    }
}

fn eval_apply(node: Apply, context: EvaluationContext) -> Result<Value> {
    let f = eval_ctx(expect_child(node.lambda())?, context.clone())?;
    let arg = eval_ctx(expect_child(node.value())?, context)?;
    f.materialize()?.call(arg)
}

fn eval_ident(node: Ident, context: EvaluationContext) -> Result<Value> {
    let ident = node.as_str();
    if let Some(v) = context.get(ident) {
        Ok(v.to_owned())
    } else {
        Err(EvalError::UnresolvedIdent(ident.to_string().into()))
    }
}

fn eval_select(node: Select, context: EvaluationContext) -> Result<Value> {
    let set = eval_ctx(expect_child(node.set())?, context)?;
    let index = expect_child(expect_child(node.index())?.first_token())?;
    let index_text = index.text().to_string();
    if let Value::AttrSet(set) = set {
        if let Some(v) = set.get(&index_text) {
            Ok(v.to_owned())
        } else {
            Err(EvalError::NoSuchIndex(index_text.into()))
        }
    } else {
        Err(EvalError::TypeMismatch(
            "attribute set".into(),
            set.human_readable_type().into(),
        ))
    }
}

fn eval_list(node: rnix::types::List, context: EvaluationContext) -> Result<Value> {
    let mut v = Vector::new();
    for item in node.items() {
        v.push_back_mut(eval_ctx(item, context.clone())?);
    }
    Ok(Value::List(v))
}

fn eval_string(node: Str, context: EvaluationContext) -> Result<Value> {
    let mut s = String::new();
    for part in node.parts() {
        match part {
            StrPart::Literal(x) => s += &x,
            StrPart::Ast(x) => s += &eval_ctx(x, context.clone())?.to_string(),
        }
    }
    Ok(s.into())
}

fn eval_lambda(node: Lambda, context: EvaluationContext) -> Result<Value> {
    let arg = expect_child(expect_child(node.arg())?.first_token())?;
    let arg_s = arg.text().to_string();
    let body = expect_child(node.body())?;
    Ok(Value::Function(arg_s, context, body))
}

fn eval_paren(node: Paren, context: EvaluationContext) -> Result<Value> {
    let body = expect_child(node.inner())?;
    eval_ctx(body, context)
}

fn eval_let_in(node: LetIn, context: EvaluationContext) -> Result<Value> {
    let mut ctx = context;
    for entry in node.entries() {
        let k = expect_child(entry.key())?;
        let v = Value::Thunk(ctx.clone(), expect_child(entry.value())?);
        let mut path_vec = vec![];
        for part in k.path() {
            let token = expect_child(part.first_token())?;
            path_vec.push(token.text().to_string());
        }
        ctx = ctx.with_path(&path_vec, v);
    }
    eval_ctx(expect_child(node.body())?, ctx)
}

pub fn eval_ctx(node: SyntaxNode, context: EvaluationContext) -> Result<Value> {
    match node.kind() {
        rnix::SyntaxKind::NODE_APPLY => eval_apply(cast(node)?, context),
        rnix::SyntaxKind::NODE_ASSERT => nyi("assert"),
        rnix::SyntaxKind::NODE_KEY => nyi("key"),
        rnix::SyntaxKind::NODE_DYNAMIC => nyi("dynamic"),
        rnix::SyntaxKind::NODE_ERROR => nyi("error"),
        rnix::SyntaxKind::NODE_IDENT => eval_ident(cast(node)?, context),
        rnix::SyntaxKind::NODE_IF_ELSE => nyi("if_else"),
        rnix::SyntaxKind::NODE_SELECT => eval_select(cast(node)?, context),
        rnix::SyntaxKind::NODE_INHERIT => nyi("inherit"),
        rnix::SyntaxKind::NODE_INHERIT_FROM => nyi("inherit from"),
        rnix::SyntaxKind::NODE_STRING => eval_string(cast(node)?, context),
        rnix::SyntaxKind::NODE_STRING_INTERPOL => nyi("string interpolation"),
        rnix::SyntaxKind::NODE_LAMBDA => eval_lambda(cast(node)?, context),
        rnix::SyntaxKind::NODE_LEGACY_LET => nyi("legacy let"),
        rnix::SyntaxKind::NODE_LET_IN => eval_let_in(cast(node)?, context),
        rnix::SyntaxKind::NODE_LIST => eval_list(cast(node)?, context),
        rnix::SyntaxKind::NODE_BIN_OP => eval_bin_op(cast(node)?, context),
        rnix::SyntaxKind::NODE_OR_DEFAULT => nyi("or"),
        rnix::SyntaxKind::NODE_PAREN => eval_paren(cast(node)?, context),
        rnix::SyntaxKind::NODE_PATTERN => nyi("patterns"),
        rnix::SyntaxKind::NODE_PAT_BIND => nyi("patterns"),
        rnix::SyntaxKind::NODE_PAT_ENTRY => nyi("patterns"),
        rnix::SyntaxKind::NODE_ROOT => eval_root(cast(node)?, context),
        rnix::SyntaxKind::NODE_ATTR_SET => nyi("attribute sets"),
        rnix::SyntaxKind::NODE_KEY_VALUE => nyi("attribute sets"),
        rnix::SyntaxKind::NODE_UNARY_OP => nyi("unary operators"),
        rnix::SyntaxKind::NODE_LITERAL => eval_literal(cast(node)?),
        rnix::SyntaxKind::NODE_WITH => nyi("with"),
        _ => Err(EvalError::UnexpectedNode),
    }
}

pub fn eval(node: SyntaxNode) -> Result<Value> {
    eval_ctx(node, EvaluationContext::new())
}
