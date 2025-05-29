import type { Delete, Emphasis, InlineCode, Parent, Strong } from "mdast";
import type { Transformer } from "unified";
import { convert } from "unist-util-is";
import { visit } from "unist-util-visit";

type InlineElement = Strong | Emphasis | InlineCode | Delete;

export function remarkMergePhrasingNodes(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert(["delete", "emphasis", "inlineCode", "strong"]),
      (_node, idx = 0, parent: Parent) => {
        const node = _node as InlineElement;
        const absorbed: InlineElement[] = [];
        while (parent.children[idx + 1]?.type === node.type) {
          absorbed.push(parent.children.splice(idx + 1, 1)[0] as InlineElement);
        }
        if (node.type === "inlineCode") {
          absorbed.forEach((absorbedNode) => {
            node.value += (absorbedNode as InlineCode).value;
          });
        } else {
          node.children.push(
            ...absorbed.flatMap(
              (absorbedNode) =>
                (absorbedNode as Exclude<InlineElement, InlineCode>).children,
            ),
          );
        }
      },
    );
  };
}
