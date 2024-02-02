use fuzzy_sourcemap::paragraph::find_paragraph;

mod utils;

#[must_use]
fn test_case(case: &str, line: usize) -> Option<String> {
  let sources = utils::get_fixture();
  let source = utils::get_fixture_file(&sources, case);
  let para = find_paragraph(&*source, line);
  match para {
    Some(span) => Some(
      sources
        .with_snippet_of_span(span, |text| String::from(text))
        .unwrap(),
    ),
    None => None,
  }
}

#[test]
fn test_find_paragraph_markdown_013() {
  insta::assert_snapshot!(test_case("markdown.md", 13).unwrap());
}

#[test]
fn test_find_paragraph_markdown_017() {
  insta::assert_snapshot!(test_case("markdown.md", 17).unwrap());
}

#[test]
fn test_find_paragraph_markdown_031() {
  insta::assert_snapshot!(test_case("markdown.md", 31).unwrap());
}

#[test]
fn test_find_paragraph_markdown_054() {
  insta::assert_snapshot!(test_case("markdown.md", 54).unwrap());
}

#[test]
fn test_find_paragraph_markdown_140() {
  insta::assert_snapshot!(test_case("markdown.md", 140).unwrap());
}

#[test]
fn test_find_paragraph_markdown_138() {
  insta::assert_snapshot!(test_case("markdown.md", 138).unwrap());
}

#[test]
fn test_find_paragraph_markdown_182() {
  insta::assert_snapshot!(test_case("markdown.md", 182).unwrap());
}

#[test]
fn test_find_paragraph_markdown_318() {
  insta::assert_snapshot!(test_case("markdown.md", 318).unwrap());
}

#[test]
fn test_find_paragraph_markdown_341() {
  insta::assert_snapshot!(test_case("markdown.md", 341).unwrap());
}

#[test]
fn test_find_paragraph_markdown_352() {
  insta::assert_snapshot!(test_case("markdown.md", 352).unwrap());
}

#[test]
fn test_find_paragraph_markdown_374() {
  insta::assert_snapshot!(test_case("markdown.md", 374).unwrap());
}

#[test]
fn test_find_paragraph_markdown_393() {
  insta::assert_snapshot!(test_case("markdown.md", 393).unwrap());
}

#[test]
fn test_find_paragraph_markdown_410() {
  assert_eq!(test_case("markdown.md", 410), None);
}

#[test]
fn test_find_paragraph_empty() {
  assert_eq!(test_case("empty.md", 1), None);
}
