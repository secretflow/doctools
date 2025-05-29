import type { Code, Parent } from "mdast";
import type { Transformer } from "unified";
import { CONTINUE, SKIP, visit } from "unist-util-visit";

export function remarkSwagger(): Transformer {
  return (tree) => {
    visit(tree, "code", (_node, idx, parent: Parent) => {
      const node = _node as Code;
      if (node.lang === "swagger") {
        parent.children[idx] = {
          type: "mdxJsxFlowElement",
          name: "OpenAPIViewer",
          attributes: [
            {
              type: "mdxJsxAttribute",
              name: "schema",
              value: node.value,
            },
          ],
          children: [],
        };
        return SKIP;
      }
      return CONTINUE;
    });
  };
}
