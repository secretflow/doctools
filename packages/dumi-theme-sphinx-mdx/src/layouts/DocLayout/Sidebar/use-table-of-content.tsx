import type { IRoute } from 'dumi';
import { useAppData, useLocation } from 'dumi';
import type { IRouteMeta } from 'dumi/dist/client/theme-api/types.js';

import type { SidebarItem } from './typing.js';

import { randstring } from '~/internals/utils/string.js';

export type SitemapEntry = {
  filepath: string | null;
  title: string;
  children: SitemapEntry[];
};

export type Sitemap = SitemapEntry[];

export function useNearestTocTree():
  | {
      projectName: string;
      indexPage: string;
      sitemap: Sitemap;
    }
  | undefined {
  const { pathname } = useLocation();
  const { routes } = useAppData();
  const pathSegments = pathname.split('/');

  while (pathSegments.length) {
    const prefix = pathSegments.join('/');
    const routeInfo = routes[prefix.slice(1)] as
      | (IRoute & { meta?: IRouteMeta })
      | undefined;
    const tocTree = routeInfo?.meta?.frontmatter?.['sitemap'];
    if (routeInfo !== undefined && tocTree !== undefined) {
      return {
        sitemap: tocTree,
        indexPage: prefix || '/',
        projectName: routeInfo?.meta?.frontmatter.title || prefix,
      };
    }
    pathSegments.pop();
  }

  return undefined;
}

function toctreeToSidebarItem(entry: SitemapEntry): SidebarItem {
  return {
    key: entry.filepath || randstring(),
    selectable: entry.filepath !== null,
    title: entry.title,
    children: entry.children.map(toctreeToSidebarItem),
  };
}

export function useTableOfContent(): SidebarItem[] {
  const info = useNearestTocTree();
  if (info === undefined) {
    return [];
  }
  return [
    {
      key: info.indexPage,
      title: info.projectName,
      children: info.sitemap.map(toctreeToSidebarItem),
    },
  ];
}
