import type * as hast from 'hast';
import type * as mdxast from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { SKIP, visit } from 'unist-util-visit';

export function rehypeBetterTable(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert<mdxast.MdxJsxFlowElement>(
        (node): node is mdxast.MdxJsxFlowElement =>
          node.type === 'mdxJsxFlowElement' &&
          (<mdxast.MdxJsxFlowElement>node).name === 'table',
      ),
      (node: mdxast.MdxJsxFlowElement) => {
        node.name = 'Table';
      },
    );

    visit(
      tree,
      convert((node): node is hast.Element => {
        return node.type === 'element' && (<hast.Element>node).tagName === 'table';
      }),
      (node: hast.Element, idx, parent) => {
        const element: mdxast.MdxJsxFlowElement = {
          type: 'mdxJsxFlowElement',
          name: 'Table',
          attributes: [],
          // @ts-expect-error We are putting HAST elements in JSX
          children: node.children,
        };
        // @ts-expect-error We are putting a JSX element inside a HAST element
        parent.children[idx] = element;
        return SKIP;
      },
    );
  };
}
