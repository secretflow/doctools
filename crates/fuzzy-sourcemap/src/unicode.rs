#[cfg(test)]
mod tests {
  use swc_core::{
    common::{source_map::Pos, sync::Lrc, BytePos, FileName, SourceMap, Span, SyntaxContext},
    ecma::{
      ast::{Expr, ObjectLit},
      codegen::{self, Node},
    },
  };

  #[test]
  #[should_panic]
  fn test_char_boundary() {
    let text = "🦀🐍🦕";
    // each emoji is 3 bytes

    dbg!(text.len());
    // 13

    let sourcemap = Lrc::<SourceMap>::default();

    let source = sourcemap.new_source_file(FileName::Anon, text.to_string());

    let span = Span::new(
      // 0 reserved for dummy spans
      BytePos::from_usize(1),
      BytePos::from_usize(1 + text.trim().len()),
      SyntaxContext::empty(),
    );

    // {"🦀🐍🦕": "🦀🐍🦕"}
    // swc generates a mapping from } to the source string
    // one byte before the end of the last character for some reason
    // which will cause a panic when generating the sourcemap when
    // the last character has more than one byte
    let expr = Expr::Object(ObjectLit {
      span,
      props: vec![],
    });

    {
      let mut buffer = vec![];
      let mut mappings = vec![];
      let mut emitter = codegen::Emitter {
        cfg: codegen::Config::default().with_minify(true),
        cm: sourcemap.clone(),
        comments: None,
        wr: Box::new(codegen::text_writer::JsWriter::new(
          sourcemap.clone(),
          "\n",
          &mut buffer,
          Some(&mut mappings),
        )),
      };

      expr.emit_with(&mut emitter).unwrap();

      println!("{}", String::from_utf8_lossy(&buffer));

      dbg!(&mappings);

      assert!(mappings
        .iter()
        .all(|(pos, _)| { dbg!(source.src.is_char_boundary(dbg!(pos.to_usize() - 1))) }))
    }
  }
}
