import type * as mdast from "mdast";
import type { TextDirective } from "mdast-util-directive";
import type { MdxJsxFlowElement } from "mdast-util-mdx";
import type { Transformer } from "unified";
import { convert } from "unist-util-is";
import { SKIP, visitParents } from "unist-util-visit-parents";

/**
 * Plugin to support explicitly providing IDs for Markdown content.
 *
 * Requires remark-directive.
 *
 * Look for text directives with the name "target" and an `id` attribute:
 *
 *     :target[my-id]
 *
 * and convert them to either:
 *
 * - A JSX attribute on the next JSX element:
 *
 *   So this:
 *
 *   ```mdx
 *   :target[my-id]
 *   <h1>My Heading</h1>
 *   ```
 *
 *   becomes this:
 *
 *   ```mdx
 *   <h1 id="my-id">My Heading</h1>
 *   ```
 *
 * - A `hProperties` property on the next Markdown node, which could be processed by
 *   remark-rehype:
 *
 *   So this:
 *
 *   ```mdx
 *   :target[my-id]
 *   # My Heading
 *   ```
 *
 *   becomes this:
 *
 *   ```mdx
 *   <h1 id="my-id">My Heading</h1>
 *   ```
 *
 *   upon remark-rehype.
 */
export function remarkAttrs(): Transformer {
  return (tree) => {
    visitParents(
      tree,
      convert((node): node is TextDirective => {
        if (node.type !== "textDirective") {
          return false;
        }
        if ((node as TextDirective).name !== "target") {
          return false;
        }
        if (typeof (node as TextDirective).attributes?.["id"] !== "string") {
          return false;
        }
        return true;
      }),
      (node: TextDirective, ancestors: mdast.Parent[]) => {
        const id = node.attributes?.["id"];

        if (typeof id !== "string" || id === "") {
          return;
        }

        const parent = ancestors[ancestors.length - 1];

        let nextNode: mdast.Content | undefined = undefined;

        let currentBranch: mdast.Content = node;
        let ancestor: mdast.Parent | undefined = ancestors[ancestors.length - 1];

        while (!nextNode && ancestor) {
          const index = ancestor.children.indexOf(currentBranch);
          const nextSibling = ancestor.children.slice(index + 1).shift();
          if (nextSibling === undefined) {
            currentBranch = ancestor as mdast.Content;
            ancestor = ancestors[ancestors.indexOf(ancestor) - 1];
            continue;
          } else {
            nextNode = nextSibling;
            break;
          }
        }

        // remove textDirective from the tree
        parent.children.splice(parent.children.indexOf(node), 1);

        if (!nextNode) {
          return;
        }

        if (["mdxJsxTextElement", "mdxJsxFlowElement"].includes(nextNode.type)) {
          if (
            (nextNode as MdxJsxFlowElement).attributes.some(
              (attr) => attr.type === "mdxJsxAttribute" && attr.name === "id",
            )
          ) {
            return;
          }
          (nextNode as MdxJsxFlowElement).attributes.push({
            type: "mdxJsxAttribute",
            name: "id",
            value: id,
          });
          return;
        } else {
          nextNode.data ??= {};
          // escape hatch provided by mdast-util-to-hast
          // https://github.com/syntax-tree/mdast-util-to-hast#hproperties
          Object.assign(nextNode.data, { hProperties: { id } });
        }

        return SKIP;
      },
    );
  };
}
