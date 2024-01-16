use swc_core::{
    common::{sync::Lrc, SourceMap},
    ecma::codegen::{text_writer::JsWriter, Config, Emitter, Node},
};

pub fn print_one<N: Node>(node: &N, cm: Option<Lrc<SourceMap>>, conf: Option<Config>) -> String {
    let cm = cm.unwrap_or_else(|| Lrc::new(SourceMap::default()));
    let mut buf = vec![];
    let mut emitter = Emitter {
        cfg: conf.unwrap_or_default(),
        cm: cm.clone(),
        comments: None,
        wr: Box::new(JsWriter::new(cm, "\n", &mut buf, None)),
    };
    node.emit_with(&mut emitter).unwrap();
    String::from_utf8(buf).unwrap()
}
