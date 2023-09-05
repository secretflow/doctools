import { Tooltip } from 'antd';
import { useFullSidebarData as useDumiSidebarData } from 'dumi';
import { useMemo } from 'react';

import type { RuntimeSidebar } from '~/plugin/manifest/index.mjs';

/**
 * Generate sidebar items from Dumi routes
 */
export function useFileSystemTree(): RuntimeSidebar {
  const routes = useDumiSidebarData();

  const tree = useMemo<RuntimeSidebar>(() => {
    const items: RuntimeSidebar = [];

    Object.entries(routes).forEach(([prefix, groups]) => {
      const segments = prefix.slice(1).split('/');

      let parent = items;

      segments.forEach((segment, idx) => {
        const path = '/' + segments.slice(0, idx + 1).join('/');
        const existing = parent.find((item) => item.key === path);
        if (existing) {
          parent = existing.children ?? [];
        } else {
          const subgroup = {
            key: path,
            title: segment,
            children: [],
          };
          parent.push(subgroup);
          parent = subgroup.children;
        }
      });

      groups.forEach((group) => {
        group.children.forEach((child) => {
          const link = child['link'];
          const segment = link.split('/').pop();
          const title = <Tooltip title={child.title || 'NO TITLE'}>{segment}</Tooltip>;
          if (prefix === link) {
            return;
          }
          const existing = parent.find((item) => item.key === link);
          if (!existing) {
            parent.push({
              key: link,
              title: title,
              children: [],
            });
          } else if (!existing.title) {
            existing.title = title;
          }
        });
      });

      parent.sort((a, b) => a.key.localeCompare(b.key));
    });

    return items;
  }, [routes]);

  return [{ key: '/', title: '(root)', children: tree }];
}
