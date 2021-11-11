use std::{env::VarError, rc::Rc};

use rpds::HashTrieMap;
use thiserror::Error;

use crate::{evaluator::EvalError, value::Value, ErrorString};

mod definitions;

#[derive(Error, Debug)]
pub enum BuiltinError {
    #[error("Aborted: {0}")]
    Aborted(String),
    #[error("Error thrown: {0}")]
    Thrown(String),
    #[error("Type mismatch - expected {0}, found {1}")]
    TypeMismatch(ErrorString, ErrorString),
    #[error("Builtin not yet implemented: {0}")]
    NotYetImplemented(ErrorString),
    #[error("An error occurred parsing a version string (passed {0} and {1})")]
    VersionParse(ErrorString, ErrorString),
    #[error("version_compare produced unexpected output")]
    UnexpectedVersionOutput,
    #[error("Index {0} was out of bounds")]
    OutOfBounds(i64),
    #[error("Unknown hash type {0}")]
    UnknownHash(ErrorString),
    #[error("Attribute setting missing required attribute {0}")]
    MissingAttr(ErrorString),
    #[error("The from and to arguments to replaceStrings must be the same length")]
    ReplaceStringsArgLength,
    #[error("Cannot serialize {0} to string")]
    CannotSerialize(ErrorString),

    #[error("An error occurred fetching the environment variable {0}")]
    Environment(ErrorString, #[source] VarError),

    #[cfg(feature = "json")]
    #[error(transparent)]
    JSON(#[from] serde_json::Error),

    #[cfg(feature = "regex")]
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

type Result = std::result::Result<Value, EvalError>;

fn nyi<S: Into<ErrorString>>(builtin: S) -> Result {
    Err(BuiltinError::NotYetImplemented(builtin.into()).into())
}

fn mismatch<T>(expected: &'static str, received: Value) -> std::result::Result<T, EvalError> {
    Err(BuiltinError::TypeMismatch(expected.into(), received.human_readable_type().into()).into())
}

pub fn builtins_set() -> Value {
    let mut s = HashTrieMap::new();
    fn add<F: 'static + Fn(Value) -> Result>(
        s: &mut HashTrieMap<String, Value>,
        name: &'static str,
        f: F,
    ) {
        s.insert_mut(name.to_string(), Value::BuiltinFunction(Rc::new(f)));
    }

    add(&mut s, "derivation", definitions::derivation);
    add(&mut s, "abort", definitions::abort);
    add(&mut s, "add", definitions::add);
    add(&mut s, "all", definitions::all);
    add(&mut s, "any", definitions::any);
    add(&mut s, "attrNames", definitions::attr_names);
    add(&mut s, "attrValues", definitions::attr_values);
    add(&mut s, "baseNameOf", definitions::base_name_of);
    add(&mut s, "bitAnd", definitions::bit_and);
    add(&mut s, "bitOr", definitions::bit_or);
    add(&mut s, "bitXor", definitions::bit_xor);
    add(&mut s, "catAttrs", definitions::cat_attrs);
    add(&mut s, "ceil", definitions::ceil);
    add(&mut s, "compareVersions", definitions::compare_versions);
    add(&mut s, "concatLists", definitions::concat_lists);
    add(&mut s, "concatMap", definitions::concat_map);
    add(&mut s, "concatStringsSep", definitions::concat_strings_sep);
    add(&mut s, "deepSeq", definitions::deep_seq);
    add(&mut s, "dirOf", definitions::dir_of);
    add(&mut s, "div", definitions::div);
    add(&mut s, "elem", definitions::elem);
    add(&mut s, "elemAt", definitions::elem_at);
    add(&mut s, "fetchGit", definitions::fetch_git);
    add(&mut s, "fetchTarball", definitions::fetch_tarball);
    add(&mut s, "fetchurl", definitions::fetchurl);
    add(&mut s, "filter", definitions::filter);
    add(&mut s, "filterSource", definitions::filter_source);
    add(&mut s, "floor", definitions::floor);
    add(&mut s, "foldl'", definitions::foldl);
    add(&mut s, "fromJSON", definitions::from_json);
    add(&mut s, "functionArgs", definitions::function_args);
    add(&mut s, "genList", definitions::gen_list);
    add(&mut s, "getAttr", definitions::get_attr);
    add(&mut s, "getEnv", definitions::get_env);
    add(&mut s, "hasAttr", definitions::has_attr);
    add(&mut s, "hashFile", definitions::hash_file);
    add(&mut s, "hashString", definitions::hash_string);
    add(&mut s, "head", definitions::head);
    add(&mut s, "import", definitions::import);
    add(&mut s, "intersectAttrs", definitions::intersect_attrs);
    add(&mut s, "isAttrs", definitions::is_attrs);
    add(&mut s, "isBool", definitions::is_bool);
    add(&mut s, "isFloat", definitions::is_float);
    add(&mut s, "isFunction", definitions::is_function);
    add(&mut s, "isInt", definitions::is_int);
    add(&mut s, "isList", definitions::is_list);
    add(&mut s, "isNull", definitions::is_null);
    add(&mut s, "isPath", definitions::is_path);
    add(&mut s, "isString", definitions::is_string);
    add(&mut s, "length", definitions::length);
    add(&mut s, "lessThan", definitions::less_than);
    add(&mut s, "listToAttrs", definitions::list_to_attrs);
    add(&mut s, "map", definitions::map);
    add(&mut s, "mapAttrs", definitions::map_attrs);
    add(&mut s, "match", definitions::f_match);
    add(&mut s, "mul", definitions::mul);
    add(&mut s, "parseDrvName", definitions::parse_drv_name);
    add(&mut s, "partition", definitions::partition);
    add(&mut s, "path", definitions::path);
    add(&mut s, "pathExists", definitions::path_exists);
    add(&mut s, "placeholder", definitions::placeholder);
    add(&mut s, "readDir", definitions::read_dir);
    add(&mut s, "readFile", definitions::read_file);
    add(&mut s, "removeAttrs", definitions::remove_attrs);
    add(&mut s, "replaceStrings", definitions::replace_strings);
    add(&mut s, "seq", definitions::seq);
    add(&mut s, "sort", definitions::sort);
    add(&mut s, "split", definitions::split);
    add(&mut s, "splitVersion", definitions::split_version);
    add(&mut s, "storePath", definitions::store_path);
    add(&mut s, "stringLength", definitions::string_length);
    add(&mut s, "sub", definitions::sub);
    add(&mut s, "substring", definitions::substring);
    add(&mut s, "tail", definitions::tail);
    add(&mut s, "throw", definitions::throw);
    add(&mut s, "toFile", definitions::to_file);
    add(&mut s, "toJSON", definitions::to_json);
    add(&mut s, "toPath", definitions::to_path);
    add(&mut s, "toString", definitions::to_string);
    add(&mut s, "toXML", definitions::to_xml);
    add(&mut s, "trace", definitions::trace);
    add(&mut s, "tryEval", definitions::try_eval);
    add(&mut s, "typeOf", definitions::type_of);

    Value::AttrSet(s)
}

pub fn base_context() -> HashTrieMap<String, Value> {
    let mut s = HashTrieMap::new();
    let builtins = builtins_set();
    fn add<F: 'static + Fn(Value) -> Result>(
        s: &mut HashTrieMap<String, Value>,
        name: &'static str,
        f: F,
    ) {
        s.insert_mut(name.to_string(), Value::BuiltinFunction(Rc::new(f)));
    }

    s.insert_mut("builtins".to_string(), builtins);
    add(&mut s, "derivation", definitions::derivation);
    add(&mut s, "import", definitions::import);

    s
}
