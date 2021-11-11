use std::borrow::Cow;

pub type ErrorString = Cow<'static, str>;

pub mod builtins;

pub mod evaluator;

#[cfg(feature = "serde")]
pub mod serde;

pub mod value;
