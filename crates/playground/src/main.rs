use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
  bytes: Vec<u8>,
}

fn main() {
  dbg!(serde_json::from_str::<Test>(r#"{"bytes": "abc"}"#).unwrap());
}
