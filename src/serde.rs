use std::convert::TryInto;

use rpds::{HashTrieMap, Vector};
use serde::{
    de::{self, Visitor},
    ser::{Error, SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::value::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::String(x) => serializer.serialize_str(x),
            Value::Integer(x) => serializer.serialize_i64(*x),
            Value::Floating(x) => serializer.serialize_f64(*x),
            Value::Path(x) => serializer.serialize_str(x),
            Value::Boolean(x) => serializer.serialize_bool(*x),
            Value::Null => serializer.serialize_unit(),
            Value::Function(_, _, _) => Err(S::Error::custom("cannot serialize functions")),
            Value::AttrSet(x) => {
                let mut map = serializer.serialize_map(Some(x.size()))?;
                for (k, v) in x {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::List(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for v in x {
                    seq.serialize_element(v)?;
                }
                seq.end()
            }
            Value::Thunk(_, _) => Serialize::serialize(
                &self
                    .to_owned()
                    .materialize_deep()
                    .map_err(S::Error::custom)?,
                serializer,
            ),
            Value::BuiltinFunction(_) => Err(S::Error::custom("cannot serialize functions")),
        }
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(v.try_into().map_err(E::custom)?)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut v = Vector::new();
        while let Some(x) = seq.next_element()? {
            v.push_back_mut(x);
        }
        Ok(Value::List(v))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut m = HashTrieMap::new();
        while let Some((k, v)) = map.next_entry()? {
            m.insert_mut(k, v);
        }
        Ok(Value::AttrSet(m))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}
