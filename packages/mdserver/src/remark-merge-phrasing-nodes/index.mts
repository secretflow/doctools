import type { Strong, Emphasis, InlineCode, Delete, Parent } from 'mdast';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { visit } from 'unist-util-visit';

type InlineElement = Strong | Emphasis | InlineCode | Delete;

export function remarkMergePhrasingNodes(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert<InlineElement>(['delete', 'emphasis', 'inlineCode', 'strong']),
      <T extends InlineElement>(node: T, idx: number, parent: Parent) => {
        const absorbed: T[] = [];
        while (parent.children[idx + 1]?.type === node.type) {
          absorbed.push(parent.children.splice(idx + 1, 1)[0] as T);
        }
        if (node.type === 'inlineCode') {
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
