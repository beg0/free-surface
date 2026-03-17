//! # Configuration Value
//!
use super::parse_helpers::unquote_single;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
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

pub fn parse_value(raw_value: &str, kind: &DicoType, nargs: usize) -> Result<ConfigValue, String> {
    let raw_value_list: Vec<&str> = parse_list(raw_value, kind, nargs);

    if nargs == 1 {
        parse_single_value(raw_value, kind)
    } else {
        parse_collection_values(raw_value_list, kind)
    }
}

fn parse_list<'a>(raw_value: &'a str, kind: &DicoType, nargs: usize) -> Vec<&'a str> {
    let mut splitted = raw_value.split(';');

    // I don't understand why, but sometimes lists are separated with coma ','
    // instead of semi-column.
    // Only try the ',' if we really target a list and this is a list of words
    // otherwise, we may experiences issues with some figures in French
    // (coma is used to separate integer & decimal part in French)
    if (raw_value.find(';').is_none()) && (*kind == DicoType::String) && (nargs != 1) {
        splitted = raw_value.split(',');
    }

    let mut ret: Vec<&'a str> = splitted.map(|s| s.trim()).collect();

    let ret_last_idx = ret.len() - 1;

    let surrounded_by = |c| {
        ret[0].starts_with(c)
            && !ret[0].ends_with(c)
            && !ret[ret_last_idx].starts_with(c)
            && ret[ret_last_idx].ends_with(c)
    };

    if (nargs != 1) && !ret.is_empty() && (surrounded_by('\'') || surrounded_by('"')) {
        ret[0] = &ret[0][1..];
        let last_elt_len = ret[ret_last_idx].len();
        ret[ret_last_idx] = &ret[ret_last_idx][..(last_elt_len - 1)];
    }

    ret
}

/// Parse a boolean with every possible alternative keywords
/// both in French and English
fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.to_lowercase().as_str() {
        "vrai" | "oui" | "true" | "yes" | "1" | "on" => Ok(true),
        "faux" | "non" | "false" | "no" | "0" | "off" => Ok(false),
        _ => Err(format!("'{}' is not a valid boolean", raw)),
    }
}

pub fn parse_single_value(raw: &str, kind: &DicoType) -> Result<ConfigValue, String> {
    match kind {
        DicoType::Logical => parse_bool(raw).map(ConfigValue::Boolean),
        DicoType::Integer => i64::from_str(raw)
            .map(ConfigValue::Integer)
            .map_err(|_| format!("'{}' is not a valid integer", raw)),
        DicoType::Real => f64::from_str(raw)
            .map(ConfigValue::Float)
            .map_err(|_| format!("'{}' is not a valid float", raw)),
        // DicoType::Path => {
        //     let path = unquote_single(raw);
        //     Ok(Value::Path(std::path::PathBuf::from(path)))
        // }
        DicoType::String => Ok(ConfigValue::String(unquote_single(raw).to_string())),
    }
}

pub fn parse_collection_values(
    raw_value_list: Vec<&str>,
    kind: &DicoType,
) -> Result<ConfigValue, String> {
    match kind {
        DicoType::Logical => {
            let mut converted_values: Vec<bool> = Vec::with_capacity(raw_value_list.len());
            let mut invalid_values: Vec<&str> = Vec::new();
            for entry in raw_value_list {
                match parse_bool(entry) {
                    Ok(val) => converted_values.push(val),
                    Err(_) => invalid_values.push(entry),
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::BooleanCollection(converted_values))
            } else {
                Err(format!(
                    "'{}' are not valid booleans",
                    invalid_values.join(", ")
                ))
            }
        }
        DicoType::Integer => {
            let mut converted_values: Vec<i64> = Vec::with_capacity(raw_value_list.len());
            let mut invalid_values: Vec<&str> = Vec::new();
            for entry in raw_value_list {
                match i64::from_str(entry) {
                    Ok(val) => converted_values.push(val),
                    Err(_) => invalid_values.push(entry),
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::IntegerCollection(converted_values))
            } else {
                Err(format!(
                    "'{}' are not valid integers",
                    invalid_values.join(", ")
                ))
            }
        }
        DicoType::Real => {
            let mut converted_values: Vec<f64> = Vec::with_capacity(raw_value_list.len());
            let mut invalid_values: Vec<&str> = Vec::new();
            for entry in raw_value_list {
                match f64::from_str(entry) {
                    Ok(val) => converted_values.push(val),
                    Err(_) => invalid_values.push(entry),
                }
            }
            if invalid_values.is_empty() {
                Ok(ConfigValue::FloatCollection(converted_values))
            } else {
                Err(format!(
                    "'{}' are not valid floats",
                    invalid_values.join(", ")
                ))
            }
        }
        // DicoType::Path => {
        //     Ok(Value::PathCollection(
        //         raw_value_list.iter()
        //         .map(|raw| std::path::PathBuf::from(unquote_single(raw)))
        //         .collect()))
        // }
        DicoType::String => Ok(ConfigValue::StringCollection(
            raw_value_list
                .iter()
                .map(|raw| unquote_single(raw).to_string())
                .collect(),
        )),
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
                    other => Err(format!("element {i} is not a {}: {:?}", stringify!($scalar), other)),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(ConfigValue::$collection),
            )+
            other => Err(format!("cannot collect a Vec of {:?}", other)),
        }
    };
}

macro_rules! impl_into_scalars {
    ($self:expr, $( $collection:ident => $scalar:ident ),+ $(,)?) => {
        match $self {
            $( ConfigValue::$collection(v) => Ok(v.into_iter().map(ConfigValue::$scalar).collect()), )+
            other => Err(format!("{:?} is not a collection variant", other)),
        }
    };
}

impl ConfigValue {
    pub fn collect(values: Vec<ConfigValue>) -> Result<ConfigValue, String> {
        let first = values.first().ok_or("cannot collect an empty Vec")?;
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

#[cfg(test)]
mod tests;

// Ignore french word used in telemac
// cSpell:ignore vrai faux oui non
