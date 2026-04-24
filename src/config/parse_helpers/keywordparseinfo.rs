//! # Keyword content and location
use crate::config::configvalue::DicoType;
use crate::config::textloc::TextLoc;

/// A token in a config file, with it's location
#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    pub token: String,
    pub start_pos: TextLoc,
    pub end_pos: TextLoc,
}

/// A keyword in a config file, a.k.a a { key, value(s) } pair
#[derive(Debug, Clone)]
pub struct KeywordParseInfo {
    pub key: TokenInfo,
    pub values: Vec<TokenInfo>,
}

/// Clone the accessor with a new text value
fn fork_token_info(ti: &TokenInfo, new_text: &str) -> TokenInfo {
    TokenInfo {
        token: new_text.to_owned(),
        start_pos: ti.start_pos.clone(),
        end_pos: ti.end_pos.clone(),
    }
}

impl KeywordParseInfo {
    pub fn keyname(&self) -> &String {
        &self.key.token
    }

    pub fn valuenames(&self) -> Vec<&String> {
        self.values.iter().map(|v| &v.token).collect()
    }

    pub fn fix_list(&mut self, kind: &DicoType, nargs: usize) {
        if (self.values.len() == 1) && (*kind == DicoType::String) && (nargs != 1) {
            let single_value = &self.values[0];
            let splitted = single_value.token.split(',');
            let mut new_values: Vec<TokenInfo> = Vec::new();
            for e in splitted {
                new_values.push(fork_token_info(single_value, e));
            }
            self.values = new_values;
        }
    }

    pub fn fixed_list_values(&self, kind: &DicoType, nargs: usize) -> Vec<TokenInfo> {
        if (self.values.len() == 1) && (*kind == DicoType::String) && (nargs != 1) {
            let single_value = &self.values[0];
            let splitted = single_value.token.split(',');
            let mut ret: Vec<TokenInfo> = Vec::new();
            for e in splitted {
                ret.push(fork_token_info(single_value, e));
            }
            ret
        } else {
            self.values.clone()
        }
    }
}
