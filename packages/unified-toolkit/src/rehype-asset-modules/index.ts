import * as path from 'node:path';
import * as url from 'node:url';

import { fromJs } from 'esast-util-from-js';
import type { Image, Parent, Root } from 'mdast';
import type { MdxjsEsm, MdxJsxFlowElement, MdxJsxTextElement } from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { SKIP, visit } from 'unist-util-visit';

/**
 * Collect all local assets and generate ESM import statements for them, such
 * that Webpack will pick them up and copy them to the output directory.
 *
 * Provide a regex in options to limit the types of assets to be discovered.
 * The default is to discover .png, .tiff, .jpg, .jpeg, and .svg files.
 *
 * Currently this plugin look for possible assets in the following places:
 *
 * - Markdown images (e.g. `![alt](./image.png)`)
 * - The `src` attribute of any JSX elements
 *   (e.g. `<CustomFigure src="./figure.png" />`)
 * - <a href="..." download={true} />
 *
 * No other attributes are supported at the moment. To utilize Webpack for including
 * assets referenced by e.g. <a href="..." download />, write it like you would with
 * any normal asset modules.
 *
 * ```mdx
 * import pdf from './document.pdf';
 *
 * # Download <a href={pdf} download>PDF</a>
 * ```
 *
 * Markdown notations like `![alt](./image.png)` are converted to JSX syntax
 * because the variable introduced by the import statement can only take effect
 * in JSX context.
 *
 * Additionally, the `download` attribute on downloadable links, if set to `true`,
 * will be replaced with the filename, so that the file can be downloaded with the
 * correct filename even after bundling.
 *
 * (Surely someone must've already done this? How do Next.js and Gatsby handle
 * this?)
 *
 * Replacement for https://github.com/umijs/dumi/blob/06c903b0e7c66a655429870fdab3b4e760695ae3/src/loaders/markdown/transformer/rehypeImg.ts#L23
 */
export function rehypeAssetModules(
  { test }: { test: Pick<RegExp, 'test'> } = { test: /\.(png|tiff|jpe?g|svg)$/ },
): Transformer {
  return (tree) => {
    type ImportInfo = {
      variable: string;
      path: string;
    };

    const refs: ImportInfo[] = [];

    function maybeLocalAsset(assetPath: string): string | null {
      try {
        const maybeURL = new url.URL(assetPath);
        if (maybeURL.protocol !== 'file:') {
          // Not a local asset
          return null;
        }
      } catch {
        // Not a valid WHATWG URL
      }
      if (!test.test(assetPath)) {
        return null;
      }
      if (!assetPath.startsWith('./') && !assetPath.startsWith('../')) {
        return `./${assetPath}`;
      }
      return assetPath;
    }

    function modifyJSXElement(node: MdxJsxFlowElement | MdxJsxTextElement) {
      let attr: MdxJsxFlowElement['attributes'][number] | undefined;

      if (node.name === 'a') {
        // process download link
        attr = node.attributes.find((a) => 'name' in a && a.name === 'href');
      } else {
        attr = node.attributes.find((a) => 'name' in a && a.name === 'src');
      }

      if (
        // No available attribute
        attr === undefined ||
        // is a spread expression like {...attrs}
        attr.type === 'mdxJsxExpressionAttribute' ||
        // is a JSX expression like <MyComponent src={...} />
        // This could be something like <MyComponent src={"./image.png"} />
        // but if it is written like that then the author should know what
        // they are doing.
        typeof attr.value === 'object' ||
        attr.value === undefined
      ) {
        return;
      }

      const ref = maybeLocalAsset(attr.value);

      if (ref === null) {
        return;
      }

      const variable = `asset${refs.length}`;
      const info: ImportInfo = { path: ref, variable };

      refs.push(info);

      const idx = node.attributes.findIndex((a) => a === attr);

      node.attributes[idx] = {
        type: 'mdxJsxAttribute',
        name: attr.name,
        value: {
          type: 'mdxJsxAttributeValueExpression',
          value: info.variable,
          data: { estree: fromJs(info.variable, { module: true }) },
        },
      };

      // modify the `download` attribute if necessary

      const download = node.attributes.find(
        (a) => a.type === 'mdxJsxAttribute' && a.name === 'download',
      );
      if (
        download &&
        download.value &&
        typeof download.value !== 'string' &&
        download.value.value === JSON.stringify(true)
      ) {
        download.value = path.basename(ref);
      }
    }

    visit(tree, 'image', (node: Image, idx: number, parent: Parent) => {
      if (maybeLocalAsset(node.url) === null) {
        return;
      }
      const imgElem: MdxJsxTextElement = {
        type: 'mdxJsxTextElement',
        name: 'img',
        attributes: [{ type: 'mdxJsxAttribute', name: 'src', value: node.url }],
        children: [],
      };
      if (typeof node.alt === 'string') {
        imgElem.attributes.push({
          type: 'mdxJsxAttribute',
          name: 'alt',
          value: node.alt,
        });
      }
      // This is then processed by the next traversal
      parent.children[idx] = imgElem;
      return SKIP;
    });

    visit(
      tree,
      convert<MdxJsxFlowElement | MdxJsxTextElement>(
        (node): node is MdxJsxFlowElement | MdxJsxTextElement => {
          if (node.type !== 'mdxJsxFlowElement' && node.type !== 'mdxJsxTextElement') {
            return false;
          }
          if (
            (<MdxJsxFlowElement>node).name === 'a' &&
            (<MdxJsxFlowElement>node).attributes.some(
              (a) => 'name' in a && a.name === 'download',
            )
          ) {
            return true;
          }
          const idx = (<MdxJsxFlowElement>node).attributes.findIndex(
            (attr) => 'name' in attr && attr.name === 'src',
          );
          if (idx === -1) {
            return false;
          }
          return true;
        },
      ),
      modifyJSXElement,
    );

    const root = tree as Root;

    return {
      ...root,
      children: [
        ...refs.map((ref): MdxjsEsm => {
          const stmt = `import ${ref.variable} from ${JSON.stringify(ref.path)};`;
          return {
            type: 'mdxjsEsm',
            value: stmt,
            data: { estree: fromJs(stmt, { module: true }) },
          };
        }),
        ...root.children,
      ],
    };
  };
}
