import type * as mdast from "mdast";
import type * as mdxast from "mdast-util-mdx";
import type { Transformer } from "unified";
import { convert } from "unist-util-is";
import { visit } from "unist-util-visit";

/**
 * Plugin to fix invalid DOM nesting in MDAST.
 *
 * Currently this supports the following:
 *
 * - Remove remark-gfm autolinks that happen to be inside JSX anchors.
 */
export function remarkValidateDOMNesting(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert((node, _idx, parent): node is mdast.Link => {
        if (!parent) {
          return false;
        }
        if (node.type !== "link") {
          return false;
        }
        const parentIsJSXAnchor =
          ["mdxJsxTextElement", "mdxJsxFlowElement"].includes(parent.type) &&
          ["a", "Link"].includes(String((parent as mdxast.MdxJsxFlowElement).name));
        return parentIsJSXAnchor;
      }),
      (node, idx = 0, parent: mdast.Parent) => {
        parent.children.splice(idx, 1, ...(node as mdast.Link).children);
        return idx;
      },
    );
  };
}
