use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "name")]
pub enum JSXElement {
  Intrinsic(String),
  Ident(String),
  Fragment,
}

fn main() {
  println!(
    "{}",
    serde_json::to_string_pretty(&JSXElement::Ident("div".into())).unwrap()
  );

  let t = "impl Feature for Struct;";

  t.split_inclusive(['\n', '<', '>', '<', '>'])
    .for_each(|chunk| match chunk.chars().last() {
      Some(c) => {
        println!("{} {}", c, chunk);
      }
      None => unreachable!(),
    })
}
