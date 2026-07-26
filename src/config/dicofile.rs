//! # Telemac dico file
//!
//! Dictionaries contains the list of all keywords allowed for steering files
//! (a.k.a "cas" file) for a given program (Telemac2D, Telemac3D, Artemis,
//!  Tomawac...).

mod dicokeyword;
mod parser;

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub use dicokeyword::{ChoiceValidationError, DicoKeyword, GuiControl};
pub use parser::{parse, parse_file};

/// Possibles locales in a Dico file
const LOCALES: [&str; 2] = ["en", "fr"];

type ErrorPtr = Box<dyn std::error::Error>;
type VecErrorPtr = Vec<ErrorPtr>;

// All keywords for a given locale, indexed by their (normalized) name
type DicoInner = HashMap<String, Rc<DicoKeyword>>;

/// Telemac's Dico - all possible keyword (in every language) allowed in a steering file
///
/// The content of the dico depend on which Telemac program is run (Telemac2D,
/// Telemac3D, Artemis, Tomawac...).
///
/// Dico can created with [parse_dico].
pub struct Dico {
    /// Each keywords, indexed per locale
    per_locale: HashMap<String, DicoInner>,
}

impl Dico {
    pub fn get(&self, name: &str) -> Option<&DicoKeyword> {
        let normalized_name = normalize_keyword_name(name);

        for inner in self.per_locale.values() {
            if let Some(keyword) = inner.get(&normalized_name) {
                return Some(keyword);
            }
        }
        None
    }

    /// Iterator visiting all keywords of the dico
    ///
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        let english_locale = self.per_locale.get(LOCALES[0]);

        match english_locale {
            Some(keywords) => Iter {
                iter: keywords.iter(),
            },
            None => Iter {
                iter: std::collections::hash_map::Iter::default(),
            },
        }
    }

    /// Returns the number of keywords in the dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use free_surface::config::dicofile::parse;
    ///
    /// let dico = parse("").unwrap();
    /// assert_eq!(dico.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        let first = self.per_locale.iter().next();
        match first {
            Some((_locale, keywords)) => keywords.len(),
            None => 0,
        }
    }

    /// Returns `true` if the dictionary contains no keywords.
    ///
    /// # Examples
    ///
    /// ```
    /// use free_surface::config::dicofile::parse;
    ///
    /// let dico = parse("").unwrap();
    /// assert!(dico.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        let first = self.per_locale.iter().next();
        match first {
            Some((_locale, keywords)) => keywords.is_empty(),
            None => true,
        }
    }
}

impl fmt::Debug for Dico {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let locales: Vec<&String> = self.per_locale.keys().collect();
        let first_locale = self.per_locale.get(LOCALES[0]).unwrap();

        f.debug_struct("Dico")
            .field("locales", &locales)
            .field("keywords", &first_locale)
            .finish()
    }
}

pub struct Iter<'a> {
    //  base: &'a DicoInner,
    iter: std::collections::hash_map::Iter<'a, String, Rc<DicoKeyword>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a DicoKeyword);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, val) = self.iter.next()?;
        Some((key, val))
    }
}

fn normalize_keyword_name(name: &str) -> String {
    name.trim().to_uppercase()
}
