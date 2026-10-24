//! # Support of UNITS() and WITH_UNITS() in fluent translation files

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_bundle::{FluentArgs, FluentValue};

use super::lookup_with_args;

fn units_helper<'a>(
    positional: &[FluentValue<'a>],
    text_id_prefix: &'static str,
) -> FluentValue<'a> {
    let Some(cnt) = positional.first() else {
        return FluentValue::Error;
    };

    if let Some(FluentValue::String(unit)) = positional.get(1) {
        let text_id = format!("{text_id_prefix}-{unit}");

        let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::with_capacity(1);
        args.insert(Cow::Borrowed("cnt"), cnt.clone());

        FluentValue::String(
            lookup_with_args(&fluent_i18n::get_locale(), text_id.as_str(), &args).into(),
        )
    } else {
        cnt.clone()
    }
}

/// Implementation of the `UNITS()` function in fluent (*.flt) files
///
/// This function is indented to be added to fluent bundle with [fluent_bundle::FluentBundle::add_function]
///
/// # Parameters
/// - `positional`: List of positional parameter of the `UNITS()` function.
///   The `UNITS()` function takes 2 positional arguments: `cnt` and `unit`
/// - `_named`: Map of named arguments, the `UNITS()` function does not take any named arguments
///
pub fn units<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    units_helper(positional, "unit")
}

/// Implementation of the `UNITS()` function in fluent (*.flt) files
///
/// This function is indented to be added to fluent bundle with [fluent_bundle::FluentBundle::add_function]
///
/// # Parameters
/// - `positional`: List of positional parameter of the `WITH_UNITS()` function.
///   The `WITH_UNITS()` function takes 2 positional arguments: `cnt` and `unit`
/// - `_named`: Map of named arguments, the `WITH_UNITS()` function does not take any named arguments
///
pub fn with_units<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
    units_helper(positional, "with-unit")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_units<F>(cnt: usize, unit: &'static str, callback: F) -> FluentValue<'static>
    where
        F: Fn(
            &[fluent_message::FluentValue<'static>],
            &fluent_message::FluentArgs<'_>,
        ) -> FluentValue<'static>,
    {
        let positional = [
            FluentValue::Number(cnt.into()),
            FluentValue::String(unit.into()),
        ];
        let named = FluentArgs::new();
        callback(&positional, &named)
    }

    #[test]
    fn unit_singular_byte() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(1, "byte", units);
        assert!(matches!(value, FluentValue::String(v) if v == "byte"));
    }

    #[test]
    fn unit_plural_byte() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(2, "byte", units);
        assert!(matches!(value, FluentValue::String(v) if v == "bytes"));
    }

    #[test]
    fn unit_singular_u32() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(1, "u32", units);
        assert!(matches!(value, FluentValue::String(v) if v == "word of 32-bits"));
    }

    #[test]
    fn unit_plural_u32() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(2, "u32", units);
        assert!(matches!(value, FluentValue::String(v) if v == "words of 32-bits"));
    }

    #[test]
    fn with_unit_singular_byte() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(1, "byte", with_units);
        assert!(matches!(value, FluentValue::String(v) if v == "1 byte"));
    }

    #[test]
    fn with_unit_plural_byte() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(2, "byte", with_units);
        assert!(matches!(value, FluentValue::String(v) if v == "2 bytes"));
    }

    #[test]
    fn with_unit_singular_u32() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(1, "u32", with_units);
        assert!(matches!(value, FluentValue::String(v) if v == "1 word of 32-bits"));
    }

    #[test]
    fn with_unit_plural_u32() {
        crate::set_locale(Some("C".into())).expect("LANG=C failed");

        let value = call_units(2, "u32", with_units);
        assert!(matches!(value, FluentValue::String(v) if v == "2 words of 32-bits"));
    }
}
