#[derive(Debug, PartialEq)]
pub struct PackageImport<'a> {
  pub package: &'a str,
  pub path: &'a str,
}

/// Check if a string is a bare module specifier, per [Node.js specification]
///
/// See also [deno_core::resolve_import].
///
/// [Node.js specification]: https://nodejs.org/api/esm.html#import-specifiers
pub fn is_bare_specifier(specifier: &str) -> bool {
  match url::Url::parse(specifier) {
    Err(url::ParseError::RelativeUrlWithoutBase)
      if !(specifier.starts_with('/')
        || specifier.starts_with("./")
        || specifier.starts_with("../")) =>
    {
      true
    }
    Ok(_) => false,
    Err(_) => false,
  }
}

/// Try to parse a string as an ES module import specifier containing a package name
/// and an optional path.
///
/// See [validate-npm-package-name] and the Node.js [module resolution algorithm].
///
/// This function assumes that the specifier is already a valid bare specifier.
/// The validity of the package name is not checked.
/// Query strings and fragments, if present, are included in the path as-is.
///
/// [validate-npm-package-name]: https://www.npmjs.com/package/validate-npm-package-name
/// [module resolution algorithm]: https://nodejs.org/api/esm.html#resolution-algorithm-specification
pub fn parse_specifier(specifier: &str) -> Option<PackageImport> {
  let mut slashes = if specifier.starts_with('@') { 2 } else { 1 };
  let mut start_of_path = specifier.len();
  for (i, c) in specifier.char_indices() {
    if c == '/' {
      slashes -= 1;
    }
    if slashes == 0 {
      start_of_path = i;
      break;
    }
  }
  if start_of_path == 0 {
    None
  } else {
    Some(PackageImport {
      package: &specifier[..start_of_path],
      path: &specifier[start_of_path..],
    })
  }
}

#[cfg(test)]
mod test_specifier {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  fn test_package_is_bare_specifier() {
    assert!(is_bare_specifier("foo"));
  }

  #[test]
  fn test_package_with_subpackage_is_bare_specifier() {
    assert!(is_bare_specifier("foo/bar"));
  }

  #[test]
  fn test_http_url_is_not_bare_specifier() {
    assert!(!is_bare_specifier("http://foo.com"));
  }

  #[test]
  fn test_file_url_is_not_bare_specifier() {
    assert!(!is_bare_specifier("file:///foo"));
  }

  #[test]
  fn test_relative_url_is_not_bare_specifier_1() {
    assert!(!is_bare_specifier("/foo"));
  }

  #[test]
  fn test_relative_url_is_not_bare_specifier_2() {
    assert!(!is_bare_specifier("./foo"));
  }

  #[test]
  fn test_relative_url_is_not_bare_specifier_3() {
    assert!(!is_bare_specifier("../foo"));
  }

  #[test]
  fn test_valid_url_is_not_bare_specifier() {
    assert!(!is_bare_specifier(
      "postgresql+psycopg2://scott:tiger@localhost/test"
    ));
  }

  #[test]
  fn test_malformed_url_is_bare_specifier() {
    // Node considers this a bare specifier
    assert!(is_bare_specifier("  ://    /   "));
  }

  #[test]
  fn test_parse_import_react() {
    match parse_specifier("react") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "react");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_example_com() {
    match parse_specifier("example.com") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "example.com");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_under_score() {
    match parse_specifier("under_score") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "under_score");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_npm_thingy() {
    match parse_specifier("@npm/thingy") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "@npm/thingy");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_jane_foo_js() {
    match parse_specifier("@jane/foo.js") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "@jane/foo.js");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_react_dom_client() {
    match parse_specifier("react-dom/client") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "react-dom");
        assert_eq!(path, "/client");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_mdx_js_mdx_lib_compiler() {
    match parse_specifier("@mdx-js/mdx/lib/compiler.js") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "@mdx-js/mdx");
        assert_eq!(path, "/lib/compiler.js");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_lodash_es_first() {
    match parse_specifier("lodash-es/first.js") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "lodash-es");
        assert_eq!(path, "/first.js");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_package_only() {
    match parse_specifier("my-package") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "my-package");
        assert_eq!(path, "");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_package_with_path() {
    match parse_specifier("my-package/path/to/file.js") {
      Some(PackageImport { package, path }) => {
        assert_eq!(package, "my-package");
        assert_eq!(path, "/path/to/file.js");
      }
      None => panic!(),
    }
  }

  #[test]
  fn test_parse_import_empty_string() {
    assert!(parse_specifier("").is_none());
  }

  #[test]
  fn test_parse_import_single_slash() {
    assert!(parse_specifier("/").is_none());
  }

  #[test]
  fn test_parse_import_multiple_slashes() {
    assert!(parse_specifier("///").is_none());
  }
}
