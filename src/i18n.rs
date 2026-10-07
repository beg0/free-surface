//! # Localisation management

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_bundle::FluentValue;
use fluent_i18n::i18n;

use fluent_i18n::fluent_templates::{LanguageIdentifier, Loader};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use unic_langid::langid;

i18n!("resources/translations", fallback = "en-US");

/// Set locale to use in free-surface
///
/// Same as [fluent_i18n::set_locale] but with locale negotiation
pub fn set_locale(requested: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let default = langid!("en-US");

    let req_or_sys_locale = requested.or_else(sys_locale::get_locale);

    let lang_id: LanguageIdentifier = if let Some(req) = req_or_sys_locale {
        if req == "C" {
            default.clone()
        } else {
            req.parse()?
        }
    } else {
        default.clone()
    };

    let available: Vec<LanguageIdentifier> = LOCALES.locales().cloned().collect();

    let negotiated = negotiate_languages(
        &[lang_id],
        &available,
        Some(&default),
        NegotiationStrategy::Filtering,
    );

    // negotiate_languages() always returns at least the default
    fluent_i18n::set_locale(Some(&negotiated[0].to_string()))?;

    Ok(())
}

/// Look up `text_id` for `lang` in Fluent.
pub fn lookup(lang: &LanguageIdentifier, text_id: &str) -> String {
    LOCALES.lookup(lang, text_id)
}

/// Look up `text_id` for `lang` with `args` in Fluent.
pub fn lookup_with_args(
    lang: &LanguageIdentifier,
    text_id: &str,
    args: &HashMap<Cow<'static, str>, FluentValue>,
) -> String {
    LOCALES.lookup_with_args(lang, text_id, args)
}

/// Macro to lookup a translation for a given key.
///
/// # Usage
///
/// This macro can be used in two ways:
///
/// 1. `t!("key")`
///
///   Looks up the translation for the given key in the current locale
///
/// 2. `t!("key", { arg1 => value1, arg2 => value2, ... })`
///
///   Looks up the translation for the given key in the current locale
///   and replaces the placeholders in the translation with the provided values.
///
///   The argument values must implement the [`ToFluentValue`] trait, which allows
///   converting various types to a [`FluentValue`].
///
/// # Using a custom static loader
///
/// If you don't want to use the [`i18n!`] macro or have a custom static loader,
/// you can use it with this macro by passing it as the first argument as follows:
///
/// ```rust,ignore
/// use fluent_templates::Loader;
/// use fluent_i18n::t;
///
/// let custom_loader = fluent_templates::static_loader! {
///     static CUSTOM_LOCALES = {
///         locales: "path/to/locales",
///         fallback_language: "en-US",
///         customise: |bundle| {
///             bundle.set_use_isolating(false);
///         }
///     };
/// };
///
/// let translation = t!(CUSTOM_LOCALES, "key");
/// let translation_args = t!(CUSTOM_LOCALES, "key", { arg1 => value1, arg2 => value2 });
/// ```
///
/// [`ToFluentValue`]: fluent_i18n::ToFluentValue
/// [`FluentValue`]: fluent_templates::fluent_bundle::FluentValue
///
///
/// # Debugging
///
/// When the raw mode is enabled via [`set_raw_mode`],
/// this macro will return the key itself instead of looking up the translation.
/// This is useful for debugging purposes to see which keys are being requested.
///
/// [`set_raw_mode`]: fluent_i18n::set_raw_mode
// Allow using `crate::` in the macro definition.
//
// See <https://rust-lang.github.io/rust-clippy/master/index.html#crate_in_macro_def>
//
// This is normally not recommended for macro hygiene reasons,
// but we need it here to access the `LOCALES` static loader
// from the crate where it is defined.
//
// In other words, using `$crate` points to `fluent_i18n::LOCALES`
// which is not what we want. Instead, we want to access the `LOCALES`
// from the crate where the `t` macro is invoked.
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! t {
    // t!("key")
    ($key:expr) => {
        if fluent_i18n::locale::RAW_MODE_ENABLED.with(|b| *b.borrow()) {
            $key.to_string()
        } else {
            $crate::i18n::lookup(&fluent_i18n::get_locale(), $key)
        }

    };

    // t!("key", { arg => val, ... })
    ($key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        if fluent_i18n::locale::RAW_MODE_ENABLED.with(|b| *b.borrow()) {
            $key.to_string()
        } else {
            use fluent_i18n::ToFluentValue;
            use std::borrow::Cow;
            let mut args = ::std::collections::HashMap::new();
            $(
                args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
            )+
            $crate::i18n::lookup_with_args(&fluent_i18n::get_locale(), $key, &args)
        }
    }};

    // t!(LOCALES, "key")
    ($locales:expr, $key:expr) => {{
        if fluent_i18n::locale::RAW_MODE_ENABLED.with(|b| *b.borrow()) {
            $key.to_string()
        } else {
            use fluent_i18n::fluent_templates::Loader;
            $locales.lookup(&fluent_i18n::get_locale(), $key)
        }
    }};

    // t!(LOCALES, "key", { arg => val, ... })
    ($locales:expr, $key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        if fluent_i18n::locale::RAW_MODE_ENABLED.with(|b| *b.borrow()) {
            $key.to_string()
        } else {
            use fluent_i18n::fluent_templates::Loader;
            use fluent_i18n::ToFluentValue;
            use std::borrow::Cow;
            let mut args = ::std::collections::HashMap::new();
            $(
                args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
            )+
            $locales.lookup_with_args(&fluent_i18n::get_locale(), $key, &args)
        }
    }};
}

// cSpell:ignore customise
