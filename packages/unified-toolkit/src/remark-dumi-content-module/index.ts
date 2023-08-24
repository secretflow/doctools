import { fromJs } from 'esast-util-from-js';
import type { BlockContent, Parent, Root, YAML } from 'mdast';
import type { MdxjsEsm } from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

type AmbientComponent = {
  specifier: string;
  source: string;
};

type HoistedNodes = MdxjsEsm | YAML;

/**
 * @deprecated
 *
 * Wrap the entire document in a <DumiPage /> component.
 *
 * The resulting document will be given to @mdx-js/loader to be compiled
 * into a standalone ES module exporting a React component (which is the page).
 *
 * Warning: all ESM import/export statements will be hoisted to the top of the
 * document, which may cause problems if the document contains any side effects.
 * import/export statements cannot be inside the <DumiPage /> component
 * otherwise the resulting module will not be a valid ES module.
 *
 * All ambient ("builtin") components will be imported at the top of the file.
 *
 * Ideally this is not necessary, and Dumi would use @mdx-js/react and
 * MDXProvider and any global components at the top-level of the component tree,
 * but alas.
 *
 * Replacement for https://github.com/umijs/dumi/blob/1fae04da8c697bebe20cbd0c81b754223df9aaf0/src/loaders/markdown/index.ts#L115-L136
 */
export function remarkDumiContentModule(
  {
    builtins,
  }: {
    builtins: Record<string, AmbientComponent>;
  } = { builtins: {} },
): Transformer {
  return (tree) => {
    const hoistedNodes: HoistedNodes[] = Object.values(builtins).map(
      ({ specifier, source }) => {
        const stmt = `import ${specifier} from ${JSON.stringify(source)};`;
        return {
          type: 'mdxjsEsm',
          value: stmt,
          data: {
            estree: fromJs(stmt, { module: true }),
          },
        };
      },
    );

    // Not typing tree as Root and not using the parametrized form of visit
    // visit<Root, 'mdxjsEsm'> because tsc will complain about excessively deep
    // type instantiation due to how MDAST has been typed.
    visit(tree, 'mdxjsEsm', (node: MdxjsEsm, idx: number, parent: Parent) => {
      hoistedNodes.push(node);
      parent.children.splice(idx, 1);
      return idx; // keep traversing children[idx] which is now the next node
    });

    // Bump Frontmatter to the top of the document
    // Technically at this stage it doesn't matter where the frontmatter is
    // because it's already been parsed and extracted, but in case this is used
    // with remark-stringify then Frontmatter has to be at the top.
    visit(tree, 'yaml', (node: YAML, idx: number, parent: Parent) => {
      hoistedNodes.unshift(node);
      parent.children.splice(idx, 1);
      return idx; // keep traversing children[idx] which is now the next node
    });

    const root = tree as Root;

    const wrapped: Root = {
      type: 'root',
      children: [
        ...hoistedNodes,
        {
          type: 'mdxjsEsm',
          value: "import { DumiPage } from 'dumi';",
          data: {
            estree: fromJs("import { DumiPage } from 'dumi';", {
              module: true,
            }),
          },
        },
        {
          type: 'mdxJsxFlowElement',
          name: 'DumiPage',
          attributes: [],
          children: root.children as BlockContent[],
        },
      ],
    };

    return wrapped;
  };
}
