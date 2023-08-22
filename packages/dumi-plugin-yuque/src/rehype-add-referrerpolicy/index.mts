import type { Element } from 'hast';
import type { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

export function rehypeAddReferrerpolicy(): Transformer {
  return (tree) => {
    visit(tree, 'element', (node: Element) => {
      {
        /* 添加 referrerpolicy=no-referrer
        解决访问语雀图片 referrer acl 限制问题 */
      }
      if (node.tagName === 'img') {
        if (!node.properties) {
          node.properties = {};
          node.properties['referrerPolicy'] = 'no-referrer';
        } else {
          if (!node.properties['referrerPolicy']) {
            node.properties['referrerPolicy'] = 'no-referrer';
          }
        }
      }
    });
  };
}
