use swc_utils::jsx::{builder::DocumentBuilder, factory::JSXElement};

fn main() {
  let mut builder = DocumentBuilder::new(Default::default());
  builder
    .element(&JSXElement::Fragment, |e| e, None)
    .enter(&["children"])
    .element(
      &"div".into(),
      |e| e.prop("title", "Document".into(), None),
      None,
    )
    .enter(&["children"])
    .value("The quick brown fox jumps over the lazy dog.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Lorem ipsum dolor sit amet, consectetur adipiscing elit.".into())
    .value("Sed non risus.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Cras elementum ultrices diam.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Maecenas ligula massa, varius a, semper congue, euismod non, mi.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat.".into())
    .element(&JSXElement::Intrinsic("br".into()), |e| e, None)
    .value("Duis semper.".into())
    .element(&JSXElement::Intrinsic("section".into()), |e| e, None)
    .enter(&["children"])
    .element(&JSXElement::Intrinsic("h1".into()), |e| e, None)
    .value("Heading".into())
    .element(&JSXElement::Intrinsic("p".into()), |e| e, None)
    .value("Lorem ipsum dolor sit amet, consectetuer adipiscing elit.".into())
    .element(&JSXElement::Intrinsic("p".into()), |e| e, None)
    .value("Aliquam tincidunt mauris eu risus.".into())
    .flush();
  let snippet = builder.declare();
  println!("{}", serde_json::to_string_pretty(&snippet).unwrap())
}
