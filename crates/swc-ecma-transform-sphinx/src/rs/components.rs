use swc_ecma_utils2::tag_whitelist;

tag_whitelist!(
  pub enum Transformed {
    Paragraph,
    Strong,
    Emphasis,
    Code,
    CodeBlock,
    Math,
    HorizontalRule,
    Span,
    Raw,
  }
);
