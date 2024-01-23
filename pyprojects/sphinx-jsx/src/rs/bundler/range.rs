use core::fmt;
use std::ops;

use pyo3::FromPyObject;
use swc_core::common::{BytePos, SourceFile, Span};

macro_rules! one_indexed {
  ($num:expr) => {
    if $num == 0 {
      panic!("{} should start at 1", stringify!($num));
    }
  };
}

/// https://microsoft.github.io/monaco-editor/docs.html#classes/Range.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
  /// 1-indexed
  line_number: usize,
  /// 1-indexed
  column: usize,
}

/// https://microsoft.github.io/monaco-editor/docs.html#classes/Range.html

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
  /// 1-indexed
  start_line_number: usize,
  /// 1-indexed
  start_column: usize,
  /// 1-indexed
  end_line_number: usize,
  /// 1-indexed, exclusive
  end_column: usize,
}

#[derive(Debug, Clone, FromPyObject)]
pub enum SourcePosition {
  Lines((usize, usize)),
  Range((usize, usize, usize, usize)),
}

impl SourcePosition {
  pub fn reify(&self, source_file: &SourceFile) -> Option<Span> {
    match self {
      SourcePosition::Lines((start, end)) => {
        let range = lines_to_range(source_file, *start, *end)?;
        range_to_span(source_file, range)
      }
      SourcePosition::Range(range) => {
        let range = Range::from(*range);
        range_to_span(source_file, range)
      }
    }
  }
}

impl fmt::Display for SourcePosition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SourcePosition::Lines((start, end)) => f.write_fmt(format_args!("L{}-L{}", start, end)),
      SourcePosition::Range((start_line, start_column, end_line, end_column)) => {
        f.write_fmt(format_args!(
          "L{}:{}-L{}:{}",
          start_line, start_column, end_line, end_column
        ))
      }
    }
  }
}

impl Range {
  pub fn new(
    start_line_number: usize,
    start_column: usize,
    end_line_number: usize,
    end_column: usize,
  ) -> Self {
    one_indexed!(start_line_number);
    one_indexed!(start_column);
    one_indexed!(end_line_number);
    one_indexed!(end_column);
    assert!(start_line_number <= end_line_number);
    if start_line_number == end_line_number {
      assert!(start_column <= end_column);
    }
    Self {
      start_line_number,
      start_column,
      end_line_number,
      end_column,
    }
  }
}

impl ops::BitOr for Range {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    let (x, y) = self.into();
    let (z, w) = rhs.into();
    let mut points = vec![x, y, z, w];
    points.sort();
    let upper_left = points.remove(0);
    let lower_right = points.pop().unwrap();
    (upper_left, lower_right).into()
  }
}

impl Into<(Position, Position)> for Range {
  fn into(self) -> (Position, Position) {
    (
      Position {
        line_number: self.start_line_number,
        column: self.start_column,
      },
      Position {
        line_number: self.end_line_number,
        column: self.end_column,
      },
    )
  }
}

impl From<(Position, Position)> for Range {
  fn from((lhs, rhs): (Position, Position)) -> Self {
    Self {
      start_line_number: lhs.line_number,
      start_column: lhs.column,
      end_line_number: rhs.line_number,
      end_column: rhs.column,
    }
  }
}

impl From<(usize, usize, usize, usize)> for Range {
  fn from(index: (usize, usize, usize, usize)) -> Self {
    Self::new(index.0, index.1, index.2, index.3)
  }
}

macro_rules! one_indexed {
  ($num:expr) => {
    if $num == 0 {
      panic!("{} should start at 1", stringify!($num));
    }
  };
}

pub fn line_to_range(source_file: &SourceFile, line_number: usize) -> Option<Range> {
  one_indexed!(line_number);
  let line_idx = line_number - 1;
  if line_idx >= source_file.count_lines() {
    return None;
  }
  let (start, end) = source_file.line_bounds(line_idx);
  Some(Range {
    start_line_number: line_number,
    start_column: 1,
    end_line_number: line_number,
    end_column: (end.0 - start.0) as usize,
  })
}

pub fn lines_to_range(
  source_file: &SourceFile,
  start_line_number: usize,
  end_line_number: usize,
) -> Option<Range> {
  assert!(start_line_number <= end_line_number);
  let lhs = line_to_range(source_file, start_line_number)?;
  let rhs = line_to_range(source_file, end_line_number)?;
  Some(lhs | rhs)
}

pub fn position_to_bytepos(source_file: &SourceFile, position: Position) -> Option<BytePos> {
  one_indexed!(position.line_number);
  one_indexed!(position.column);
  let line_idx = position.line_number - 1;
  let column_idx = position.column - 1;
  if line_idx >= source_file.count_lines() {
    return None;
  }
  let (start, end) = source_file.line_bounds(line_idx);
  let result = start.0 as usize + column_idx;
  if result >= end.0 as usize || result > u32::MAX as usize {
    return None;
  }
  Some(BytePos(result as u32))
}

pub fn range_to_span(source_file: &SourceFile, range: Range) -> Option<Span> {
  let start = position_to_bytepos(
    source_file,
    Position {
      line_number: range.start_line_number,
      column: range.start_column,
    },
  )?;
  let end = position_to_bytepos(
    source_file,
    Position {
      line_number: range.end_line_number,
      column: range.end_column,
    },
  )?;
  Some((start, end).into())
}

#[cfg(test)]
mod tests {
  use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};

  use super::{line_to_range, range_to_span};

  fn make_source_files() -> (Lrc<SourceMap>, Lrc<SourceFile>, Lrc<SourceFile>) {
    let sourcemap: Lrc<SourceMap> = Default::default();

    let main = sourcemap.new_source_file(
      FileName::Custom("main.jsx".into()),
      r#"// 01 'use strict';
// 02 import React from 'react';
// 03 import ReactDOM from 'react-dom';
// 04 import App from './App';
// 05 import './index.css';
// 06
// 07 ReactDOM.render(
// 08   <React.StrictMode>
// 09     <App />
// 10   </React.StrictMode>,
// 11   document.getElementById('root')
// 12 );
        "#
      .into(),
    );

    let app = sourcemap.new_source_file(
      FileName::Custom("App.jsx".into()),
      r#"// 01 'use strict';
// 02 import React from 'react';
// 03 import logo from './logo.svg';
// 04 import './App.css';
// 05
// 06 function App() {
// 07   return (
// 08     <div className="App">
// 09       <header className="App-header">
// 10         <img src={logo} className="App-logo" alt="logo" />
// 11         <p>
// 12           Edit <code>src/App.jsx</code> and save to reload.
// 13         </p>
// 14         <a
// 15           className="App-link"
// 16           href="https://reactjs.org"
// 17           target="_blank"
// 18           rel="noopener noreferrer"
// 19         >
// 20           Learn React
// 21         </a>
// 22       </header>
// 23     </div>
// 24   );
// 25 }
// 26
// 27 export default App;
      "#
      .into(),
    );

    (sourcemap, main, app)
  }

  #[test]
  fn test_range_union() {
    let lhs = super::Range::from((1, 1, 1, 10));
    let rhs = super::Range::from((1, 5, 1, 15));
    let expected = super::Range::from((1, 1, 1, 15));
    assert_eq!(lhs | rhs, expected);
  }

  #[test]
  fn test_range_union_disjoint_inverted() {
    let lhs = super::Range::from((5, 1, 5, 15));
    let rhs = super::Range::from((1, 3, 1, 6));
    let expected = super::Range::from((1, 3, 5, 15));
    assert_eq!(lhs | rhs, expected);
  }

  #[test]
  fn test_extract_one_line() {
    let (sourcemap, _, app) = make_source_files();
    let range = line_to_range(&app, 8).unwrap();
    let span = range_to_span(&app, range).unwrap();
    sourcemap
      .with_snippet_of_span(span, |text| {
        assert_eq!(text, r#"// 08     <div className="App">"#);
      })
      .unwrap();
  }

  #[test]
  fn test_extract_multiple_lines() {
    let (sourcemap, _, app) = make_source_files();
    let range = super::Range::from((11, 1, 14, 1));
    let span = range_to_span(&app, range).unwrap();
    sourcemap
      .with_snippet_of_span(span, |text| {
        assert_eq!(
          text,
          r#"// 11         <p>
// 12           Edit <code>src/App.jsx</code> and save to reload.
// 13         </p>
"#
        )
      })
      .unwrap();
  }
}
