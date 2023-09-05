import * as path from 'node:path';
import * as url from 'node:url';

import type { IRoute } from 'dumi';
import type * as mdast from 'mdast';
import type * as mdxast from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { SKIP, visit } from 'unist-util-visit';

type Supported = mdast.Link | mdxast.MdxJsxTextElement | mdxast.MdxJsxFlowElement;

/**
 * Replace all relative links pointing to local Markdown files with react-router
 * <Link />s and all absolute URLs with <a target="_blank" />s.
 *
 * The following cases are supported:
 *
 * - Relative links that correctly resolves to a Markdown file on the filesystem
 *   (if you can click it in VSCode and it opens the correct file then it's supported)
 *
 * The following cases are NOT supported at the moment:
 *
 * - Links with extensions omitted
 * - Links with path variables
 * - Tabbed pages
 * - Implicit `index.md` pages
 *
 * Replacement for https://github.com/umijs/dumi/blob/7e250d8039ad59e4dea969313dfc8f012b2fb0df/src/loaders/markdown/transformer/rehypeLink.ts#L19
 */
export function rehypeReactRouter(
  {
    routes,
  }: {
    routes: Record<string, IRoute>;
  } = { routes: {} },
): Transformer {
  return async (tree, file) => {
    const pathMapping: Record<string, string> = {};

    Object.values(routes).map(async (route) => {
      if (!route.file) {
        return;
      }
      pathMapping[route.file] = route.absPath;
    });

    visit(
      tree,
      convert<Supported>(['link', 'mdxJsxTextElement', 'mdxJsxFlowElement']),
      (node: Supported, idx: number, parent: mdast.Parent) => {
        let href: string;

        if (node.type === 'link') {
          href = node.url;
        } else {
          const attr = node.attributes.find(
            (x) => (<mdxast.MdxJsxAttribute>x).name === 'href',
          );
          if (!attr || typeof attr.value !== 'string') {
            return;
          }
          href = attr.value;
        }

        if (!href) {
          return;
        }

        if (href.startsWith('#')) {
          return;
        }

        let parsed: url.URL;

        try {
          parsed = new url.URL(href, 'file://');
        } catch {
          // malformed or unprocessable URL
          return;
        }

        let title: string | null | undefined = undefined;
        if (node.type === 'link' && node.title) {
          title = node.title;
        }

        if (parsed.protocol !== 'file:') {
          // External link
          if (
            node.type === 'link' ||
            (node.type === 'mdxJsxTextElement' && node.name === 'a')
          ) {
            // replace with <a target="_blank" />
            parent.children[idx] = {
              type: 'mdxJsxTextElement',
              name: 'a',
              attributes: [
                { type: 'mdxJsxAttribute', name: 'href', value: href.toString() },
                { type: 'mdxJsxAttribute', name: 'target', value: '_blank' },
                {
                  type: 'mdxJsxAttribute',
                  name: 'rel',
                  value: 'noopener',
                },
              ],
              children: node.children,
            };
            if (title) {
              (<mdxast.MdxJsxTextElement>parent.children[idx]).attributes.push({
                type: 'mdxJsxAttribute',
                name: 'title',
                value: title,
              });
            }
            return SKIP;
          } else {
            // some other elements, do nothing
            return;
          }
        } else {
          // Internal link, absolute or relative

          // make absolute
          const resolved = path.resolve(file.path, '..', href);
          parsed = new url.URL(resolved, 'file://');

          let linkTo: string | undefined = undefined;

          linkTo = pathMapping[parsed.pathname];

          if (linkTo !== undefined) {
            if (parsed.hash) {
              linkTo = linkTo + parsed.hash;
            }
            if (
              node.type === 'link' ||
              (node.type === 'mdxJsxTextElement' && node.name === 'a')
            ) {
              // replace with <Link />
              parent.children[idx] = {
                type: 'mdxJsxTextElement',
                name: 'Link',
                attributes: [{ type: 'mdxJsxAttribute', name: 'to', value: linkTo }],
                children: node.children,
              };
              if (title) {
                (<mdxast.MdxJsxTextElement>parent.children[idx]).attributes.push({
                  type: 'mdxJsxAttribute',
                  name: 'title',
                  value: title,
                });
              }
              return SKIP;
            } else {
              // some other elements, replace the href attribute
              const attr = node.attributes.find(
                (x) => (<mdxast.MdxJsxAttribute>x).name === 'href',
              );
              if (!attr || typeof attr.value !== 'string') {
                return;
              }
              attr.value = linkTo;
              return;
            }
          }

          return;
        }
      },
    );
  };
}
