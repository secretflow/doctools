import type { Parent } from "mdast";
import type { TextDirective } from "mdast-util-directive";
import type { MdxJsxFlowElement, MdxJsxTextElement } from "mdast-util-mdx";
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
export function remarkTarget(): Transformer {
  return (tree) => {
    visitParents(
      tree,
      convert((node): node is TextDirective => {
        if (node.type !== "textDirective") {
          return false;
        }
        const directive = node as TextDirective;
        if (directive.name !== "target") {
          return false;
        }
        if (typeof directive.attributes?.["id"] !== "string") {
          return false;
        }
        return true;
      }),
      (node, ancestors: Parent[]) => {
        const id = node.attributes?.["id"];

        if (typeof id !== "string" || id === "") {
          return;
        }

        const parent = ancestors[ancestors.length - 1];

        let nextNode = undefined;

        let currentBranch = node;
        let ancestor = ancestors[ancestors.length - 1];

        while (!nextNode && ancestor) {
          const index = ancestor.children.indexOf(currentBranch);
          const nextSibling = ancestor.children.slice(index + 1).shift();
          if (nextSibling === undefined) {
            currentBranch = ancestor as TextDirective;
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
          const next = nextNode as MdxJsxTextElement | MdxJsxFlowElement;
          if (
            next.attributes.some(
              (attr) => attr.type === "mdxJsxAttribute" && attr.name === "id",
            )
          ) {
            return;
          }
          next.attributes.push({
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
