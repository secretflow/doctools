import type * as hast from "hast";
import type { MdxJsxFlowElement } from "mdast-util-mdx";
import type { Transformer } from "unified";
import type * as unist from "unist";
import { convert } from "unist-util-is";
import { SKIP, visit } from "unist-util-visit";

export function rehypeTable(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert(
        (node): node is MdxJsxFlowElement =>
          node.type === "mdxJsxFlowElement" &&
          (node as MdxJsxFlowElement).name === "table",
      ),
      (node) => {
        node.name = "Table";
      },
    );
    visit(
      tree,
      convert(
        (node): node is hast.Element =>
          node.type === "element" && (node as hast.Element).tagName === "table",
      ),
      (node, idx = 0, parent: unist.Parent) => {
        const element = {
          type: "mdxJsxFlowElement",
          name: "Table",
          attributes: [],
          children: node.children,
        };
        parent.children[idx] = element;
        return SKIP;
      },
    );
  };
}
