//! # Configuration Value
//!
use fluent_i18n::ToFluentValue;
use fluent_message::FluentMessage;

use super::parse_helpers::{parse_fortran_float, unquote_single, TokenInfo};
use std::fmt::{self, Debug};
use std::str::FromStr;

use crate::t;

#[derive(Debug, Clone, PartialEq, FluentMessage)]
pub enum DicoType {
    String,
    Integer,
    Real,
    Logical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Path(std::path::PathBuf),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    StringCollection(Vec<String>),
    PathCollection(Vec<std::path::PathBuf>),
    BooleanCollection(Vec<bool>),
    IntegerCollection(Vec<i64>),
    FloatCollection(Vec<f64>),
}

impl ToFluentValue for DicoType {
    fn to_fluent_value(&self) -> fluent_i18n::FluentValue<'static> {
        fluent_i18n::FluentValue::String(t!(self.msg_id()).into())
    }
}
pub fn parse_value_2<'a>(
    values: &'a Vec<TokenInfo>,
    kind: &DicoType,
    nargs: usize,
) -> Result<ConfigValue, Vec<(&'a TokenInfo, String)>> {
    // Be very permissive here
    // consider it's a scalar if and only if dico says so (indeed `nargs` came from the dico)
    // and the value to parse too
    if nargs == 1 && values.len() == 1 {
        parse_single_value_2(&values[0], kind).map_err(|e| vec![e])
    } else {
        parse_collection_values_2(values, kind)
    }
}

pub fn parse_single_value_2<'a>(
    value: &'a TokenInfo,
    kind: &DicoType,
) -> Result<ConfigValue, (&'a TokenInfo, String)> {
    let raw = value.token.as_str();
    match kind {
        DicoType::Logical => parse_bool(raw)
            .map(ConfigValue::Boolean)
            .map_err(|err| (value, err)),
        DicoType::Integer => i64::from_str(raw).map(ConfigValue::Integer).map_err(|err| {
            let msg = t!("config-value-invalid-integer", { "value" => raw.to_owned(), "reason" => err.to_string() });
            (value, msg)
        }),
        DicoType::Real => parse_fortran_float(raw)
            .map(ConfigValue::Float)
            .map_err(|err| {
                let msg = t!("config-value-invalid-real", { "value" => raw.to_owned(), "reason" => err.to_string() });
                (value, msg)
            }),
        // DicoType::Path => {
        //     let path = unquote_single(raw);
        //     Ok(Value::Path(std::path::PathBuf::from(path)))
        // }
        DicoType::String => Ok(ConfigValue::String(unquote_single(raw))),
    }
}

fn parse_collection_values_2<'a>(
    value_list: &'a Vec<TokenInfo>,
    kind: &DicoType,
) -> Result<ConfigValue, Vec<(&'a TokenInfo, String)>> {
    match kind {
        DicoType::Logical => {
            let mut converted_values: Vec<bool> = Vec::with_capacity(value_list.len());
            let mut invalid_values: Vec<(&'a TokenInfo, String)> = Vec::new();
            for entry in value_list {
                let raw = entry.token.as_str();
                match parse_bool(raw) {
                    Ok(val) => converted_values.push(val),
                    Err(msg) => invalid_values.push((entry, msg)),
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::BooleanCollection(converted_values))
            } else {
                Err(invalid_values)
            }
        }
        DicoType::Integer => {
            let mut converted_values: Vec<i64> = Vec::with_capacity(value_list.len());
            let mut invalid_values: Vec<(&'a TokenInfo, String)> = Vec::new();
            for entry in value_list {
                let raw = entry.token.as_str();
                match i64::from_str(raw) {
                    Ok(val) => converted_values.push(val),
                    Err(err) => {
                        let msg = t!("config-value-invalid-integer", { "value" => raw.to_owned(), "reason" => err.to_string()});
                        invalid_values.push((entry, msg));
                    }
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::IntegerCollection(converted_values))
            } else {
                Err(invalid_values)
            }
        }
        DicoType::Real => {
            let mut converted_values: Vec<f64> = Vec::with_capacity(value_list.len());
            let mut invalid_values: Vec<(&'a TokenInfo, String)> = Vec::new();
            for entry in value_list {
                let raw = entry.token.as_str();
                match parse_fortran_float(raw) {
                    Ok(val) => converted_values.push(val),
                    Err(err) => {
                        let msg = t!("config-value-invalid-real", { "value" => raw.to_owned(), "reason" => err.to_string() });
                        invalid_values.push((entry, msg));
                    }
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::FloatCollection(converted_values))
            } else {
                Err(invalid_values)
            }
        }
        // DicoType::Path => {
        //     Ok(Value::PathCollection(
        //         raw_value_list.iter()
        //         .map(|raw| std::path::PathBuf::from(unquote_single(raw)))
        //         .collect()))
        // }
        DicoType::String => Ok(ConfigValue::StringCollection(
            value_list
                .iter()
                .map(|entry| unquote_single(&entry.token).to_string())
                .collect(),
        )),
    }
}

/// Parse a boolean with every possible alternative keywords
/// both in French and English
fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.to_lowercase().as_str() {
        "vrai" | "oui" | "true" | ".true." | "yes" | "1" | "on" => Ok(true),
        "faux" | "non" | "false" | ".false." | "no" | "0" | "off" => Ok(false),
        _ => Err(t!("config-value-invalid-bool", { "value" => raw.to_owned()})),
    }
}

macro_rules! impl_collect {
    ($first:expr, $values:expr, $( $scalar:ident => $collection:ident ),+ $(,)?) => {
        match $first {
            $( ConfigValue::$scalar(_) => $values
                .into_iter()
                .enumerate()
                .map(|(i, v)| match v {
                    ConfigValue::$scalar(inner) => Ok(inner),
                    other => Err(t!("config-value-invalid-type", { "index" => i, "expectedType" => stringify!($scalar), "actualType" => other.to_string()})),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(ConfigValue::$collection),
            )+
            other => Err(t!("config-value-unimplemented-collection", { "actualType" => other.to_string() })),
        }
    };
}

macro_rules! impl_into_scalars {
    ($self:expr, $( $collection:ident => $scalar:ident ),+ $(,)?) => {
        match $self {
            $( ConfigValue::$collection(v) => Ok(v.into_iter().map(ConfigValue::$scalar).collect()), )+
            other => Err(t!("config-value-not-a-collection", { "type" => other.to_string()})),
        }
    };
}

impl ConfigValue {
    pub fn collect(values: Vec<ConfigValue>) -> Result<ConfigValue, String> {
        let first = values.first().ok_or(t!("config-value-collect-empty-vec"))?;
        impl_collect!(first, values,
            String  => StringCollection,
            Path    => PathCollection,
            Boolean => BooleanCollection,
            Integer => IntegerCollection,
            Float   => FloatCollection,
        )
    }

    pub fn into_scalars(self) -> Result<Vec<ConfigValue>, String> {
        impl_into_scalars!(self,
            StringCollection  => String,
            PathCollection    => Path,
            BooleanCollection => Boolean,
            IntegerCollection => Integer,
            FloatCollection   => Float,
        )
    }

    pub fn is_collection(&self) -> bool {
        match self {
            ConfigValue::String(_) => false,
            ConfigValue::Path(_) => false,
            ConfigValue::Boolean(_) => false,
            ConfigValue::Integer(_) => false,
            ConfigValue::Float(_) => false,
            ConfigValue::StringCollection(_) => true,
            ConfigValue::PathCollection(_) => true,
            ConfigValue::BooleanCollection(_) => true,
            ConfigValue::IntegerCollection(_) => true,
            ConfigValue::FloatCollection(_) => true,
        }
    }

    pub fn is_scalar(&self) -> bool {
        !self.is_collection()
    }

    /// Returns number of elements in the ConfigValue.
    ///
    /// On collection, it returns the number of elements in underling vector, on scalar, it returns `1`.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            ConfigValue::String(_) => 1,
            ConfigValue::Path(_) => 1,
            ConfigValue::Boolean(_) => 1,
            ConfigValue::Integer(_) => 1,
            ConfigValue::Float(_) => 1,
            ConfigValue::StringCollection(vec) => vec.len(),
            ConfigValue::PathCollection(vec) => vec.len(),
            ConfigValue::BooleanCollection(vec) => vec.len(),
            ConfigValue::IntegerCollection(vec) => vec.len(),
            ConfigValue::FloatCollection(vec) => vec.len(),
        }
    }

    /// Returns `true` if the ConfigValue is an empty vector.
    ///
    /// On scalar returns `false`.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        match self {
            ConfigValue::String(_) => false,
            ConfigValue::Path(_) => false,
            ConfigValue::Boolean(_) => false,
            ConfigValue::Integer(_) => false,
            ConfigValue::Float(_) => false,
            ConfigValue::StringCollection(vec) => vec.is_empty(),
            ConfigValue::PathCollection(vec) => vec.is_empty(),
            ConfigValue::BooleanCollection(vec) => vec.is_empty(),
            ConfigValue::IntegerCollection(vec) => vec.is_empty(),
            ConfigValue::FloatCollection(vec) => vec.is_empty(),
        }
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(value) => std::fmt::Display::fmt(&value, f),
            ConfigValue::Path(value) => std::fmt::Display::fmt(&value.display(), f),
            ConfigValue::Boolean(value) => std::fmt::Display::fmt(&value, f),
            ConfigValue::Integer(value) => std::fmt::Display::fmt(&value, f),
            ConfigValue::Float(value) => std::fmt::Display::fmt(&value, f),
            ConfigValue::StringCollection(vec) => write!(f, "{}", vec.join(", ")),
            ConfigValue::PathCollection(vec) => write!(
                f,
                "{}",
                vec.iter()
                    .map(|value| value.display().to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConfigValue::BooleanCollection(vec) => write!(
                f,
                "{}",
                vec.iter()
                    .map(bool::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConfigValue::IntegerCollection(vec) => write!(
                f,
                "{}",
                vec.iter()
                    .map(i64::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConfigValue::FloatCollection(vec) => write!(
                f,
                "{}",
                vec.iter()
                    .map(f64::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl ToFluentValue for ConfigValue {
    fn to_fluent_value(&self) -> fluent_i18n::FluentValue<'static> {
        fluent_i18n::FluentValue::String(self.to_string().into())
    }
}

impl From<Vec<String>> for ConfigValue {
    fn from(v: Vec<String>) -> Self {
        ConfigValue::StringCollection(v)
    }
}

impl From<Vec<std::path::PathBuf>> for ConfigValue {
    fn from(v: Vec<std::path::PathBuf>) -> Self {
        ConfigValue::PathCollection(v)
    }
}

impl From<Vec<bool>> for ConfigValue {
    fn from(v: Vec<bool>) -> Self {
        ConfigValue::BooleanCollection(v)
    }
}

impl From<Vec<i64>> for ConfigValue {
    fn from(v: Vec<i64>) -> Self {
        ConfigValue::IntegerCollection(v)
    }
}

impl From<Vec<f64>> for ConfigValue {
    fn from(v: Vec<f64>) -> Self {
        ConfigValue::FloatCollection(v)
    }
}

impl TryFrom<&ConfigValue> for serde_json::Value {
    type Error = serde_json::Error;
    fn try_from(value: &ConfigValue) -> Result<Self, Self::Error> {
        match value {
            ConfigValue::String(val) => serde_json::to_value(val),
            ConfigValue::Path(val) => serde_json::to_value(val),
            ConfigValue::Boolean(val) => serde_json::to_value(val),
            ConfigValue::Integer(val) => serde_json::to_value(val),
            ConfigValue::Float(val) => serde_json::to_value(val),
            ConfigValue::StringCollection(val) => serde_json::to_value(val),
            ConfigValue::PathCollection(val) => serde_json::to_value(val),
            ConfigValue::BooleanCollection(val) => serde_json::to_value(val),
            ConfigValue::IntegerCollection(val) => serde_json::to_value(val),
            ConfigValue::FloatCollection(val) => serde_json::to_value(val),
        }
    }
}

#[cfg(test)]
mod tests;

// Ignore french word used in telemac
// cSpell:ignore vrai faux oui non
