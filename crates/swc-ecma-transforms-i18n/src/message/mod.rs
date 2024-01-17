use serde::{Deserialize, Serialize};

pub mod jsx;

/// Represents a message to be translated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub plaintext: String,
}

pub fn is_empty_or_whitespace(str: &str) -> bool {
    str.chars().all(|c| c.is_ascii_whitespace())
}

/// Collapse whitespace according to HTML's whitespace rules.
///
/// https://infra.spec.whatwg.org/#ascii-whitespace
pub fn collapse_ascii_whitespace(str: &str) -> String {
    let mut result = String::new();
    let mut last_char = '\0';
    str.chars().for_each(|c: char| {
        if c.is_ascii_whitespace() {
            if last_char.is_ascii_whitespace() {
                ()
            } else {
                last_char = c;
                result.push(' ');
            }
        } else {
            last_char = c;
            result.push(c);
        }
    });
    result
}
