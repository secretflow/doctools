import * as fs from 'node:fs/promises';
import * as path from 'node:path';

import type { useRouteMeta, IRoute } from 'dumi';
import { fromJs } from 'esast-util-from-js';
import { slug } from 'github-slugger';
import type * as hast from 'hast';
import { toString as hastToString } from 'hast-util-to-string';
import { toString as mdastToString } from 'mdast-util-to-string';
import { titleCase } from 'title-case';
import type { Transformer } from 'unified';
import { select } from 'unist-util-select';

type IRouteMeta = ReturnType<typeof useRouteMeta>;

type ToCTreeEntry = {
  path: string | null;
  title: string;
  subpages: ToCTreeEntry[];
};

export type ToCTree = ToCTreeEntry[];

type MdxJsxFlowElementHAST = hast.ElementContentMap['mdxJsxFlowElement'];
type MdxJsxTextElementHAST = hast.ElementContentMap['mdxJsxTextElement'];

type ContentElement = MdxJsxFlowElementHAST | MdxJsxTextElementHAST | hast.Element;

export function rehypeDumiMetadataModule({
  routes = {},
  legacySearch,
}: {
  routes?: Record<string, IRoute>;
  legacySearch?: boolean;
} = {}): Transformer {
  return async (tree, file) => {
    const outline = file.data['outline'];

    const frontmatter: IRouteMeta['frontmatter'] = {
      title: '',
      toc: 'content',
      filename: path.relative(file.cwd, file.path),
    };

    Object.assign(frontmatter, file.data['frontmatter']);

    // This could come from https://github.com/mrzmmr/remark-extract-frontmatter
    Object.assign(frontmatter, file.data['frontmatter']);

    if (!frontmatter.description) {
      const firstParagraph = select(
        [
          // First <p> element (MDAST)
          'root > mdxJsxFlowElement[name="p"]',
          // First <p> element (MDAST)
          'root > mdxJsxTextElement[name="p"]',
          // First <p> element (HAST)
          'element[tagName="p"]',
        ].join(', '),
        tree,
      ) as ContentElement | null;
      let description: string;
      if (firstParagraph?.type === 'element') {
        description = hastToString(firstParagraph).trim();
      } else {
        description = mdastToString(firstParagraph).trim();
      }
      if (description) {
        frontmatter.description = description;
      }
    }

    if (!frontmatter?.title) {
      const firstHeading = outline?.[0];
      if (firstHeading?.title) {
        frontmatter.title = firstHeading.title;
      } else {
        const pathWithoutIndex = file.path.replace(/(\/index([^/]+)?)?\..*?$/, '');
        const titleSlug = slug(path.basename(pathWithoutIndex));
        frontmatter.title = titleCase(titleSlug.replace(/-+/g, ' '));
      }
    }

    const texts: { value: string; paraId: number; tocIndex?: number }[] = [];

    const toctree: ToCTreeEntry[] = frontmatter['toctree'];

    if (typeof toctree === 'object') {
      // Preprocess Sphinx toctree

      // file.path = /path/to/docs/project/index.mdx
      // cwd = /path/to/docs/project
      const cwd = path.dirname(file.path);

      // FIXME:
      // file.path is guaranteed to be a realpath
      // but routes[...].file may be under a symlinked directory

      const pathMapping: Record<string, string> = {};

      // map routes[...].file to routes[...].absPath, resolving possible symlinks
      await Promise.all(
        Object.values(routes).map(async (route) => {
          if (!route.file) {
            return;
          }
          try {
            const actualRouteFile = await fs.realpath(route.file);
            pathMapping[actualRouteFile] = route.absPath;
          } catch (e) {
            pathMapping[route.file] = route.absPath;
          }
        }),
      );

      const modifyPathInPlace = async (entry: ToCTreeEntry): Promise<void> => {
        if (entry.path !== null) {
          const sourceFile = path.join(cwd, entry.path);
          const absPath = pathMapping[sourceFile];
          if (absPath) {
            entry.path = absPath;
          }
        }
        entry.subpages.forEach(modifyPathInPlace);
      };

      toctree.forEach(modifyPathInPlace);
    }

    if (legacySearch && outline) {
      outline.forEach(({ order, content: textContent }) => {
        texts.push({
          value: textContent,
          paraId: 0,
          tocIndex: order,
        });
      });
    }

    const toc = outline
      // content not under section will have depth === 0
      ?.filter(({ depth }) => depth > 0)
      .map(({ id, depth, order, title }) => ({ id, title, depth, order }));

    const statements = [
      `export const frontmatter = ${JSON.stringify(frontmatter)};`,
      `export const toc = ${JSON.stringify(toc)};`,
      `export const demos = ${JSON.stringify({})};`,
      `export const texts = ${JSON.stringify(texts)};`,
    ];

    const metaFile: hast.Root = {
      type: 'root',
      children: statements.map((stmt) => ({
        type: 'mdxjsEsm',
        value: stmt,
        data: {
          estree: fromJs(stmt, { module: true }),
        },
      })),
    };

    return metaFile;
  };
}
