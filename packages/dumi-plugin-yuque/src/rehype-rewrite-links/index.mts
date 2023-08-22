import type { IRoute } from 'dumi';
import type * as hast from 'hast';
import type * as mdx from 'mdast-util-mdx';
import type { Transformer } from 'unified';
import { convert } from 'unist-util-is';
import { SKIP, visit } from 'unist-util-visit';

export type RouteMappingOptions = {
  /**
   * Regular expression to match the links you want to rewrite. The regex MUST have a
   * capture group, and the capture group MUST capture the route ID.
   */
  match: RegExp;
  /**
   * Function to extract the destination path from Dumi's router.
   *
   * @param match The route ID from the capture group in `match`
   * @param path The candidate path
   * @returns Whether the path is the correct destination
   */
  resolve?: (match: string, path: string) => boolean;
  routes: Record<string, IRoute>;
};

/**
 * Rewrite all <a>s to <Link>s that point to Dumi routes.
 *
 * Caveat: The resulting <Link>s are in fact HAST Elements, not MDAST PhrasingContent.
 */
export function rehypeRewriteLinks(
  { match, resolve, routes }: RouteMappingOptions = {
    match: /(.*)/,
    routes: {},
  },
): Transformer {
  return async (tree, file) => {
    visit(
      tree,
      convert((node) => {
        if (node.type !== 'element') {
          return false;
        }
        if ((node as hast.Element).tagName !== 'a') {
          return false;
        }
        if ((node as hast.Element).properties?.['download']) {
          return false;
        }
        if ((node as hast.Element).properties?.['href'] === undefined) {
          return false;
        }
        return true;
      }),
      (node, idx: number | null, parent: hast.Parent) => {
        if (idx === null) {
          return;
        }
        const href = (node as hast.Element).properties?.['href'];
        if (typeof href !== 'string') {
          return;
        }

        try {
          new URL(href, 'http://example.com');
        } catch (e) {
          return;
        }

        const { origin, pathname, search, hash } = new URL(href, 'http://example.com');
        const location = `${origin}${pathname}`.replace(/^http:\/\/example\.com/, '');
        const id = location.match(match)?.[1];

        if (id === undefined) {
          return;
        }

        const routeMap = routes;

        let route: IRoute | undefined;
        if (resolve !== undefined) {
          route = Object.values(routeMap).find((r) => resolve(id, r.path));
        } else {
          route = routeMap[id];
        }

        let anchor: mdx.MdxJsxTextElement;

        const props = (node as hast.Element).properties ?? {};

        if (route === undefined) {
          return;
        } else if (route.file === file.path) {
          // in-page fragment redirect
          props['href'] = hash;
          anchor = {
            type: 'mdxJsxTextElement',
            name: 'a',
            attributes: Object.entries(props).map(
              ([name, value]): mdx.MdxJsxAttribute => ({
                type: 'mdxJsxAttribute',
                name,
                value: String(value),
              }),
            ),
            // @ts-expect-error We are putting HAST nodes in MDX
            children: (node as hast.Element).children,
          };
        } else {
          // cross-page redirect
          delete props['href'];
          props['to'] = `${route.absPath}${search}${hash}`;
          anchor = {
            type: 'mdxJsxTextElement',
            name: 'Link',
            attributes: Object.entries(props).map(
              ([name, value]): mdx.MdxJsxAttribute => ({
                type: 'mdxJsxAttribute',
                name,
                value: String(value),
              }),
            ),
            // @ts-expect-error We are putting HAST nodes in MDX
            children: (node as hast.Element).children,
          };
        }

        // @ts-expect-error We are putting a JSX element inside a HAST
        parent.children[idx] = anchor;
        return SKIP;
      },
    );
  };
}
