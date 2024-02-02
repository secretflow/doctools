pub fn is_empty_or_whitespace(text: &str) -> bool {
  text.chars().all(|c| c.is_ascii_whitespace())
}

pub fn indentation_of(text: &str) -> &str {
  for (idx, ch) in text.char_indices() {
    if !matches!(ch, ' ' | '\t') {
      return &text[..idx];
    }
  }
  text
}
