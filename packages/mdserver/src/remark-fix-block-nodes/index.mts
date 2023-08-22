import type {
  Content,
  Parent,
  BlockContent,
  DefinitionContent,
  Root,
  PhrasingContent,
  Paragraph,
} from 'mdast';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { visit } from 'unist-util-visit';

type ElementType<T> = T extends (infer U)[] ? NonNullable<U> : T;

type ContainsOnlyBlockContent<T extends Content> = T extends Parent
  ? ElementType<T['children']> extends BlockContent | DefinitionContent
    ? T
    : never
  : never;

type BlockContentParent = Root | ContainsOnlyBlockContent<Content>;

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

export function remarkFixBlockNodes(): Transformer {
  return (tree) => {
    // Constraint 1: Most block elements content MUST only contain block elements.
    // This create implicit paragraphs for stray phrasing elements in block elements
    visit(
      tree,
      convert<BlockContentParent>([
        'blockquote',
        'containerDirective',
        'footnoteDefinition',
        'listItem',
        'mdxJsxFlowElement',
        'root',
      ]),
      (node: BlockContentParent) => {
        const reflowed: Content[] = [];

        let paragraph: Paragraph = { type: 'paragraph', children: [] };
        node.children.forEach((child) => {
          if ((PHRASING_CONTENT_TYPES as string[]).includes(child.type)) {
            paragraph.children.push(child as PhrasingContent);
          } else {
            if (paragraph['children'].length) {
              reflowed.push(paragraph);
              paragraph = { type: 'paragraph', children: [] };
            }
            reflowed.push(child);
          }
        });
        if (paragraph['children'].length) {
          reflowed.push(paragraph);
        }

        node.children = reflowed;
      },
    );
  };
}
