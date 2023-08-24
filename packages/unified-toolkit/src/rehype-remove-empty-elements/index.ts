import type * as hast from 'hast';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { visit } from 'unist-util-visit';

// FIXME: clean this up
function isEmptyNode(node: hast.Content): boolean {
  if (node.type.startsWith('mdx')) {
    return false;
  }
  if (node.type !== 'element' && node.type !== 'text') {
    return true;
  }
  if (node.type === 'text') {
    return node.value.trim() === '';
  }
  if (node.tagName === 'br') {
    return true;
  }
  if (['div', 'span', 'code', 'b', 'strong', 'em', 'i', 's'].includes(node.tagName)) {
    return node.children.every(isEmptyNode);
  }
  return false;
}

export function rehypeRemoveEmptyElements(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert(
        (node): node is hast.Element =>
          node.type === 'element' &&
          ['p', 'div'].includes((<hast.Element>node).tagName),
      ),
      (node: hast.Element, idx, parent: hast.Parent) => {
        if (node.children.every(isEmptyNode)) {
          parent.children.splice(idx, 1);
          return idx;
        }
        return;
      },
    );
  };
}
