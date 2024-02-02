use swc_core::common::{SourceFile, Span};

use crate::{
  one_indexed,
  whitespace::{indentation_of, is_empty_or_whitespace},
};

pub fn find_paragraph(src: &SourceFile, line_number: usize) -> Option<Span> {
  let line_number = one_indexed!(line_number);

  let line_number = {
    let mut current_line = line_number;
    loop {
      let Some(text) = src.get_line(current_line) else {
        break current_line;
      };
      if !is_empty_or_whitespace(&*text) {
        break current_line;
      } else {
        let Some(next_line) = current_line.checked_sub(1) else {
          break current_line;
        };
        current_line = next_line;
      }
    }
  };

  let line_content = src.get_line(line_number);

  let Some(line) = line_content else {
    return None;
  };

  let indent = indentation_of(&*line);

  fn find_nearest_indented(
    src: &SourceFile,
    start: usize,
    direction: i8,
    indentation: &str,
  ) -> usize {
    let mut current_line = start;

    let get_next_line = if direction < 0 {
      |u: usize, d: i8| u.checked_sub(d.abs() as usize)
    } else {
      |u: usize, d: i8| u.checked_add(d as usize)
    };

    loop {
      let Some(next_line) = get_next_line(current_line, direction) else {
        return current_line;
      };
      let Some(next_line_text) = src.get_line(next_line) else {
        return current_line;
      };
      let next_line_text = &*next_line_text;
      if !indentation_of(next_line_text).starts_with(indentation)
        || is_empty_or_whitespace(next_line_text)
      {
        return current_line;
      } else {
        current_line = next_line;
      }
    }
  }

  let lower = find_nearest_indented(src, line_number, -1, indent);
  let upper = find_nearest_indented(src, line_number, 1, indent);

  let (lower, _) = src.line_bounds(lower);
  let (_, upper) = src.line_bounds(upper);

  Some((lower, upper).into())
}
