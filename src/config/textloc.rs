//! # Text localisation
//!
//! TextLoc stores a position in a text file
//!

use std::convert::AsRef;
use std::convert::From;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const UNKNOWN_FILE: &str = "<unknown>";

/// A localisation in a text file
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextLoc {
    /// Filename if any, an empty string otherwise
    filename: Arc<PathBuf>,

    /// Line number in the file, starting to 1
    line: usize,

    /// Column number in the line, starting to 1. 0 if not set
    column: usize,
}

/// An error at a specific location in a text file
pub trait LocalizedError: std::error::Error {
    /// Get the filename of the error
    ///
    /// If no error, return an empty string otherwise
    fn filename(&self) -> &PathBuf;

    /// Get the line number in the file, starting to 1
    fn line(&self) -> usize;

    /// Get the column number in the line, starting to 1. 0 if not set
    fn column(&self) -> usize;
}

#[macro_export]
macro_rules! impl_localized_error {
    ($ty:ty { $($variant:ident),* }) => {
        impl $crate::config::textloc::LocalizedError for $ty {
            fn filename(&self) -> &std::path::PathBuf {
                match self {
                    $(Self::$variant {pos, ..} => pos.filename()),*
                }
            }
            fn line(&self) -> usize {
                match self {
                    $(Self::$variant {pos, ..} => pos.line()),*
                }

            }
            fn column(&self) -> usize {
                match self {
                    $(Self::$variant {pos, ..} => pos.column()),*
                }
            }
        }
    };
}

// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, Data, DeriveInput, Fields};

// #[proc_macro_derive(LocalizedErrorStuff)]
// pub fn derive_parse_error_info(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;

//     let Data::Enum(data_enum) = input.data else {
//         return syn::Error::new_spanned(name, "LocalizedError can only be derived for enums")
//             .to_compile_error()
//             .into();
//     };

//     let mut filename_arms = Vec::new();
//     let mut line_arms = Vec::new();
//     let mut column_arms = Vec::new();

//     for variant in data_enum.variants {
//         let variant_name = variant.ident;

//         match variant.fields {
//             Fields::Named(fields) => {
//                 let mut filename = None;
//                 let mut line = None;
//                 let mut column = None;

//                 for f in fields.named {
//                     let ident = f.ident.unwrap();
//                     if ident == "filename" {
//                         filename = Some(quote! { filename });
//                     } else if ident == "line" {
//                         line = Some(quote! { line });
//                     } else if ident == "column" {
//                         column = Some(quote! { column });
//                     } else if ident == "pos" {
//                         filename = Some(quote! { pos.filename() });
//                         line = Some(quote! { pos.line() });
//                         column = Some(quote! { pos.column() });
//                     }
//                 }

//                 let Some(filename_pat) = filename else {
//                     return syn::Error::new_spanned(
//                         variant_name,
//                         "missing field `filename`",
//                     )
//                     .to_compile_error()
//                     .into();
//                 };
//                 let Some(line_pat) = line else {
//                     return syn::Error::new_spanned(
//                         variant_name,
//                         "missing field `line`",
//                     )
//                     .to_compile_error()
//                     .into();
//                 };
//                 let Some(column_pat) = column else {
//                     return syn::Error::new_spanned(
//                         variant_name,
//                         "missing field `column`",
//                     )
//                     .to_compile_error()
//                     .into();
//                 };

//                 filename_arms.push(quote! {
//                     Self::#variant_name { filename: #filename_pat, .. } => filename,
//                 });
//                 line_arms.push(quote! {
//                     Self::#variant_name { line: #line_pat, .. } => line,
//                 });
//                 column_arms.push(quote! {
//                     Self::#variant_name { column: #column_pat, .. } => column,
//                 });
//             }
//             _ => {
//                 return syn::Error::new_spanned(
//                     variant_name,
//                     "ParseErrorInfo only supports enums with named fields",
//                 )
//                 .to_compile_error()
//                 .into();
//             }
//         }
//     }

//     let expanded = quote! {
//         impl LocalizedError for #name {
//             fn filename(&self) -> &str {
//                 match self {
//                     #(#filename_arms)*
//                 }
//             }

//             fn line(&self) -> usize {
//                 match self {
//                     #(#line_arms)*
//                 }
//             }

//             fn column(&self) -> &std::ops::Range<usize> {
//                 match self {
//                     #(#column_arms)*
//                 }
//             }
//         }
//     };

//     expanded.into()
// }

// Impl From with filename
//------------------------

impl<P: AsRef<Path>> From<(P, usize)> for TextLoc {
    fn from((filename, line): (P, usize)) -> Self {
        Self {
            filename: Arc::new(PathBuf::from(filename.as_ref())),
            line,
            column: 0,
        }
    }
}

impl<P: AsRef<Path>> From<(P, usize, usize)> for TextLoc {
    fn from((filename, line, column): (P, usize, usize)) -> Self {
        Self {
            filename: Arc::new(PathBuf::from(filename.as_ref())),
            line,
            column,
        }
    }
}

// Impl From without filename
//---------------------------

impl From<usize> for TextLoc {
    fn from(line: usize) -> Self {
        Self {
            filename: Arc::new(PathBuf::new()),
            line,
            column: 0,
        }
    }
}

// fmt::Display
impl fmt::Display for TextLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let native_filename = self.filename.as_os_str();

        let filename = if native_filename.is_empty() {
            String::from(UNKNOWN_FILE)
        } else {
            format!("{}", native_filename.display())
        };

        if self.column == 0 {
            write!(f, "{}:{}", filename, self.line)
        } else {
            write!(f, "{}:{}:{}", filename, self.line, self.column)
        }
    }
}

impl TextLoc {
    /// Clone this TextLoc and modify the line number
    #[allow(dead_code)]
    pub fn clone_with_line(&self, line: usize) -> TextLoc {
        TextLoc {
            filename: Arc::clone(&self.filename),
            line,
            column: 0,
        }
    }

    /// Clone this TextLoc and modify the line number
    #[allow(dead_code)]
    pub fn clone_with_line_col(&self, line: usize, column: usize) -> TextLoc {
        TextLoc {
            filename: Arc::clone(&self.filename),
            line,
            column,
        }
    }
    /// Clone this TextLoc and add an offset to the line number
    #[allow(dead_code)]
    pub fn clone_with_line_offset(&self, line_offset: usize) -> TextLoc {
        TextLoc {
            filename: Arc::clone(&self.filename),
            line: self.line + line_offset,
            column: 0,
        }
    }

    /// Clone this TextLoc and add an offset to the line number
    #[allow(dead_code)]
    pub fn clone_with_line_offset_col(&self, line_offset: usize, column: usize) -> TextLoc {
        TextLoc {
            filename: Arc::clone(&self.filename),
            line: self.line + line_offset,
            column,
        }
    }

    /// Get the filename associated with this position
    #[allow(dead_code)]
    pub fn filename(&self) -> &PathBuf {
        #[allow(clippy::explicit_auto_deref)]
        &*self.filename
    }

    /// Get the line number associated with this position
    /// Line numbers starts at 1. a 0 means anywhere in the file
    #[allow(dead_code)]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get the column in the line associated with this position
    /// 0 means no specific column is specified. It can be the full line or anywhere
    /// on the line.
    #[allow(dead_code)]
    pub fn column(&self) -> usize {
        self.column
    }
}
