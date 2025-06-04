import type { Root, Text } from "mdast";
import type { InlineMath } from "mdast-util-math";
import type { MdxJsxTextElement } from "mdast-util-mdx";
import type { Transformer } from "unified";
import { visit } from "unist-util-visit";

export function remarkEscapeMath(): Transformer<Root, Root> {
  return (tree) => {
    visit(tree, "inlineMath", function handler(node: InlineMath, idx = 0, parent) {
      switch (parent?.type) {
        case "mdxJsxTextElement":
          if ((parent as MdxJsxTextElement).name === "InlineMath") {
            return;
          } else {
            break;
          }
        default:
          break;
      }
      if (parent === undefined) {
        return;
      }
      const value = `$${node.value}$`.replaceAll(/\\\{/g, "{");
      const restored = { type: "text", value } satisfies Text;
      parent.children[idx] = restored;
    });
  };
}
