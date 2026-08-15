//! # Extra predicates
//!
//! This module implements advanced [predicates].
use predicates::prelude::Predicate;
use predicates::reflection;
use std::fmt;

use super::normalize_eol;

/// Predicate adapter that trims the variable being tested.
///
/// This is created by `pred.trim()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultilineNormalizedPredicate<P>
where
    P: Predicate<str>,
{
    p: P,
}

impl<P> Predicate<str> for MultilineNormalizedPredicate<P>
where
    P: Predicate<str>,
{
    fn eval(&self, variable: &str) -> bool {
        self.p.eval(normalize_eol(variable).as_str())
    }

    fn find_case<'a>(&'a self, expected: bool, variable: &str) -> Option<reflection::Case<'a>> {
        self.p.find_case(expected, normalize_eol(variable).as_str())
    }
}

impl<P> reflection::PredicateReflection for MultilineNormalizedPredicate<P>
where
    P: Predicate<str>,
{
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = reflection::Child<'a>> + 'a> {
        let params = vec![reflection::Child::new("predicate", &self.p)];
        Box::new(params.into_iter())
    }
}

impl<P> fmt::Display for MultilineNormalizedPredicate<P>
where
    P: Predicate<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.p.fmt(f)
    }
}

/// Adapter to normalize end-of-line in a Predicate<str>
///
/// e.g. wrapper around [super::normalize_eol]
#[allow(dead_code)]
pub fn multiline_nomalized<P>(p: P) -> MultilineNormalizedPredicate<P>
where
    P: Predicate<str>,
{
    MultilineNormalizedPredicate { p }
}
