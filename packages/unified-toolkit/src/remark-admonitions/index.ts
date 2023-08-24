import type { Parent } from 'mdast';
import type { ContainerDirective } from 'mdast-util-directive';
import type * as mdxast from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { SKIP, visit } from 'unist-util-visit';

const VALID_CONTAINER_TYPES = ['info', 'warning', 'success', 'error'];

const COMPATIBLE_TYPES = new Map<string, string>([
  ['note', 'info'],
  ['tip', 'info'],
  ['hint', 'info'],
  ['admonition', 'info'],
  ['caution', 'warning'],
  ['important', 'warning'],
  ['danger', 'error'],
]);

/**
 * Convert Markdown container directives to <Container />, which is Dumi's own
 * implementation of admonitions.
 *
 * Turns this:
 *
 * :::info{title="Info"}
 * This is an info message.
 * :::
 *
 * into this:
 *
 * <Container type="info" title="Info">
 *   This is an info message.
 * </Container>
 *
 * Replacement for https://github.com/umijs/dumi/blob/b9b3a82d94d8e04f22b05387e285488279af9d4a/src/loaders/markdown/transformer/remarkContainer.ts#L14
 */
export function remarkAdmonitions(): Transformer {
  return (tree) => {
    visit(
      tree,
      'containerDirective',
      (node: ContainerDirective, idx: number, parent: Parent) => {
        const { attributes, children } = node;
        let { name } = node;
        name = COMPATIBLE_TYPES.get(name) ?? name;
        if (!VALID_CONTAINER_TYPES.includes(name)) {
          return;
        }
        parent.children[idx] = {
          type: 'mdxJsxFlowElement',
          name: 'Container',
          attributes: [
            { type: 'mdxJsxAttribute', name: 'type', value: name },
            {
              type: 'mdxJsxAttribute',
              name: 'title',
              value: attributes?.['title'] ?? name.toUpperCase(),
            },
          ],
          children,
          data: node.data,
        };
        return SKIP;
      },
    );

    // Normalize container types
    visit(
      tree,
      convert<mdxast.MdxJsxFlowElement>(
        (node): node is mdxast.MdxJsxFlowElement =>
          node.type === 'mdxJsxFlowElement' &&
          (<mdxast.MdxJsxFlowElement>node).name === 'Container',
      ),
      (node: mdxast.MdxJsxFlowElement) => {
        const type = node.attributes.find(
          (attr) => attr.type === 'mdxJsxAttribute' && attr.name === 'type',
        );
        if (type && !VALID_CONTAINER_TYPES.includes(String(type.value))) {
          type.value = COMPATIBLE_TYPES.get(String(type.value)) ?? 'info';
        }
      },
    );
  };
}
