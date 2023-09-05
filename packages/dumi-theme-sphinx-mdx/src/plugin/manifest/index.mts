import * as fs from 'node:fs/promises';
import * as path from 'node:path';

import { normalizedDocDirs } from '@secretflow/dumi-mdx-loader-core';
import type { IApi as DumiAPI, IRoute } from 'dumi';
import { globby } from 'globby';
import YAML from 'yaml';

import { THEME_KEY } from '../index.cjs';

import type { Sidebar, SidebarItemDoc } from './schema.mjs';
import { Manifest } from './schema.mjs';

import { randstring } from '~/internals/utils/string.js';

type RuntimeSidebarItem = {
  key: string;
  title: React.ReactNode;
  selectable?: boolean; // visitable
  children?: RuntimeSidebar;
};

export type RuntimeSidebar = RuntimeSidebarItem[];

export type RuntimeManifest = { sidebar: RuntimeSidebar };

export async function manifestPlugin(api: DumiAPI) {
  api.onGenerateFiles(async () => {
    const docDirs = normalizedDocDirs(api.config.resolve.docDirs);
    const routes: Record<string, IRoute> = api.appData['routes'];

    const manifests: Record<string, RuntimeManifest> = {};

    await globby(docDirs.map(({ dir }) => `${dir}/**/manifest.yml`)).then(
      (manifestFiles) =>
        Promise.all(
          manifestFiles.map(async (manifestFile) => {
            const content = await fs.readFile(manifestFile, { encoding: 'utf-8' });
            const manifest = Manifest.parse(YAML.parse(content));

            const rootRoute = Object.values(routes).find((r) => {
              if (!r.file) {
                return false;
              }
              const p = path.parse(r.file);
              return (
                p.name === 'index' &&
                path.resolve(p.dir) === path.resolve(path.dirname(manifestFile))
              );
            });

            if (rootRoute?.file) {
              const indexPath = rootRoute.absPath;
              const sourceDir = path.dirname(rootRoute.file);

              const resolveRoute = (item: SidebarItemDoc) => {
                const targetRoute = Object.values(routes).find(
                  (r) => r.file === path.join(sourceDir, item.id),
                );
                if (!targetRoute) {
                  throw new Error(`Cannot find route for ${item.id}`);
                }
                return targetRoute.absPath;
              };

              const generateSidebar = (sidebar: Sidebar): RuntimeSidebar => {
                return sidebar.map((item) => {
                  switch (item.type) {
                    case 'doc': {
                      return {
                        key: resolveRoute(item),
                        title: item.label,
                      };
                    }
                    case 'link':
                      // external link
                      return {
                        key: item.href,
                        title: item.label,
                      };
                    case 'category': {
                      const key = item.link
                        ? resolveRoute(item.link)
                        : `:${randstring(8)}`;
                      return {
                        key,
                        title: item.label,
                        children: generateSidebar(item.items),
                      };
                    }
                  }
                });
              };
              manifests[indexPath] = { sidebar: generateSidebar(manifest.sidebar) };
            }
          }),
        ),
    );

    api.writeTmpFile({
      path: 'manifest.ts',
      content: `export const manifests = ${JSON.stringify(manifests)};`,
    });
  });

  api.addRuntimePlugin(() => `@@/plugin-${THEME_KEY}/manifest.ts`);
  api.addRuntimePluginKey(() => ['manifests']);
}
