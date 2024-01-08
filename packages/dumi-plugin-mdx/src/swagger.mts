import type { Code, Parent } from 'mdast';
import type { MdxJsxFlowElement } from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { visit, SKIP, CONTINUE } from 'unist-util-visit';

export function remarkSwagger(): Transformer {
  return (tree) => {
    visit(tree, 'code', (node: Code, idx: number, parent: Parent) => {
      if (node.lang === 'swagger') {
        parent.children[idx] = {
          type: 'mdxJsxFlowElement',
          name: 'OpenAPIViewer',
          attributes: [
            {
              type: 'mdxJsxAttribute',
              name: 'schema',
              value: node.value,
            },
          ],
          children: [],
        } satisfies MdxJsxFlowElement;
        return SKIP;
      }
      return CONTINUE;
    });
  };
}
