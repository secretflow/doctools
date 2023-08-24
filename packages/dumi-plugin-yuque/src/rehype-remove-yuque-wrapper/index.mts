import type * as hast from 'hast';
import type { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

export function rehypeRemoveDumiWrapper(): Transformer {
  return (tree) => {
    visit(tree, 'element', (node: hast.Element, _, parent: hast.Parent) => {
      if (node.properties && node.properties['typography'] === 'classic') {
        parent.children = node.children;
      }
    });
  };
}
