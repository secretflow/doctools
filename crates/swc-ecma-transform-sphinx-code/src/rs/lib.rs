pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {
  use deno_lite::{esm_source, DenoLite, ESFunction};
  use serde::Serialize;

  esm_source!(SERVER, "render-code", "../../dist/server/index.js");

  #[derive(Serialize, ESFunction)]
  struct RenderCode {
    code: String,
  }

  #[test]
  fn test_main() {
    let mut deno = DenoLite::default();
    let module = SERVER.load_into(&mut deno).unwrap();
    let result: String = deno
      .call_function(
        module,
        RenderCode {
          code: r#"
          import { codeToHtml } from "shiki";

          export async function render({ code }) {
            return await codeToHtml(code, {
              lang: "javascript",
              theme: "ayu-dark",
            });
          }
          "#
          .into(),
        },
      )
      .unwrap();
    println!("{}", result);
  }
}
