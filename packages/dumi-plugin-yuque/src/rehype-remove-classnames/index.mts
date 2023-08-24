import type { Element } from 'hast';
import type { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

export function rehypeRemoveClassNames(): Transformer {
  return (tree) => {
    visit(tree, 'element', (node: Element) => {
      delete node.properties?.['className'];
    });
  };
}
