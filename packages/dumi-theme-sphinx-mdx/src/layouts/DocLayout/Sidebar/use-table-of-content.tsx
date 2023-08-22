import type { IRoute } from 'dumi';
import { useAppData, useLocation } from 'dumi';
import type { IRouteMeta } from 'dumi/dist/client/theme-api/types.js';

import type { SidebarItem } from './typing.js';

import { randstring } from '~/internals/utils/string.js';

export type TocTreeEntry = {
  path: string | null;
  title: string;
  subpages: TocTreeEntry[];
};

export type TocTree = TocTreeEntry[];

export function useNearestTocTree():
  | {
      projectName: string;
      indexPage: string;
      tocTree: TocTree;
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
    const tocTree = routeInfo?.meta?.frontmatter?.['toctree'];
    if (routeInfo !== undefined && tocTree !== undefined) {
      return {
        tocTree,
        indexPage: prefix || '/',
        projectName: routeInfo?.meta?.frontmatter.title || prefix,
      };
    }
    pathSegments.pop();
  }

  return undefined;
}

function toctreeToSidebarItem(entry: TocTreeEntry): SidebarItem {
  return {
    key: entry.path || randstring(),
    selectable: entry.path !== null,
    title: entry.title,
    children: entry.subpages.map(toctreeToSidebarItem),
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
      children: info.tocTree.map(toctreeToSidebarItem),
    },
  ];
}
