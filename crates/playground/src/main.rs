use std::ops;

use swc_core::common::{sync::Lrc, BytePos, FileName, SourceFile, SourceMap, Span};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Position {
  /// 1-indexed
  line_number: usize,
  /// 1-indexed
  column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Range {
  /// 1-indexed
  start_line_number: usize,
  /// 1-indexed
  start_column: usize,
  /// 1-indexed
  end_line_number: usize,
  /// 1-indexed, exclusive
  end_column: usize,
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

macro_rules! one_indexed {
  ($num:expr) => {
    if $num == 0 {
      panic!("{} should start at 1", stringify!($num));
    }
  };
}

fn line_to_range(source_file: &SourceFile, line_number: usize) -> Option<Range> {
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

fn lines_to_range(
  source_file: &SourceFile,
  start_line_number: usize,
  end_line_number: usize,
) -> Option<Range> {
  assert!(start_line_number <= end_line_number);
  let lhs = line_to_range(source_file, start_line_number)?;
  let rhs = line_to_range(source_file, end_line_number)?;
  Some(lhs | rhs)
}

fn position_to_bytepos(source_file: &SourceFile, position: Position) -> Option<BytePos> {
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

fn range_to_span(source_file: &SourceFile, range: Range) -> Option<Span> {
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

fn main() {
  let sourcemap: Lrc<SourceMap> = Default::default();

  let _main = sourcemap.new_source_file(
    FileName::Custom("main.jsx".into()),
    r#"'use strict';
import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import './index.css';

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById('root')
);
    "#
    .into(),
  );

  let app = sourcemap.new_source_file(
    FileName::Custom("App.jsx".into()),
    r#"'use strict';
import React from 'react';
import logo from './logo.svg';
import './App.css';

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          Edit <code>src/App.jsx</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
}

export default App;
    "#
    .into(),
  );

  let range = lines_to_range(&app, 1, 10).unwrap();
  let span = range_to_span(&app, range).unwrap();

  sourcemap
    .with_snippet_of_span(span, |slice| println!("{}", slice))
    .unwrap();
}
