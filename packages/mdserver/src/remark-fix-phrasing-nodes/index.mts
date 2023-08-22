import type { Content, Parent, PhrasingContent } from 'mdast';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { visit } from 'unist-util-visit';

type ElementType<T> = T extends (infer U)[] ? NonNullable<U> : T;

type ContainsOnlyPhrasingContent<T extends Content> = T extends Parent
  ? ElementType<T['children']> extends PhrasingContent
    ? T
    : never
  : never;

type PhrasingContentParent = ContainsOnlyPhrasingContent<Content>;

const PHRASING_CONTENT_TYPES: PhrasingContent['type'][] = [
  'break',
  'delete',
  'emphasis',
  'footnote',
  'footnoteReference',
  'html',
  'image',
  'imageReference',
  'inlineCode',
  'link',
  'linkReference',
  'mdxJsxTextElement',
  'mdxTextExpression',
  'strong',
  'text',
  'textDirective',
  'inlineMath',
];

export function remarkFixPhrasingNodes(): Transformer {
  return (tree) => {
    // Constraint 2: Headings, paragraphs, and other phrasing elements MUST contain only
    // phrasing elements.
    // This pulls block elements out of phrasing elements and insert them as siblings.

    // unist-util-visit doesn't support postorder operations
    // here we repeatedly reflow until no more changes are made
    // this is unfortunately grossly inefficient
    let reflowed = false;

    do {
      reflowed = false;

      visit(
        tree,
        convert<PhrasingContentParent>([
          'heading',
          'paragraph',
          'delete',
          'emphasis',
          'footnote',
          'leafDirective',
          'link',
          'linkReference',
          'mdxJsxTextElement',
          'strong',
          'tableCell',
          'textDirective',
        ]),
        (node: PhrasingContentParent, idx: number, parent: Parent) => {
          const blockElementIdx = node.children.findIndex((x) => {
            return !PHRASING_CONTENT_TYPES.includes(x.type);
          });
          if (blockElementIdx !== -1) {
            parent.children.splice(
              idx + 1,
              0,
              ...node.children.splice(blockElementIdx),
            );
            reflowed = true;
          }
        },
      );
    } while (reflowed);
  };
}
