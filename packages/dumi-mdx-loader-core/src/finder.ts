import path from 'node:path';

import type { IApi as DumiAPI, IRoute } from 'dumi';
import { globbySync } from 'globby';
import pluralize from 'pluralize';

import type { DocDirConfig } from './utils.js';
import { extensionTest, normalizedDocDirs } from './utils.js';

type Options = {
  cwd: string;
  /** Extensions of the doc files to be included, without the leading dot */
  extensions: string[];
  /** The docDirs config from Dumi API */
  dirs: DocDirConfig[];
};

type RouteModifier = Parameters<DumiAPI['modifyRoutes']>[0]['fn'];

/**
 * Return a function that can be passed to modifyRoutes from Dumi API to add routes
 * for all files with the given extensions in the configured docDirs.
 *
 * @param extensions Extensions without the leading dot
 */
export function findDocs(options: () => Options): RouteModifier {
  return async (routes) => {
    const { extensions, cwd, dirs } = options();

    // .md files are already discovered by Dumi. We must remove 'md' from the list
    // to avoid duplicate routes.
    const extraExtensions = extensions.filter((ext) => ext !== 'md');

    // look for files with these extensions
    const pattern = `**/*.{${extraExtensions.map((ext) => `${ext},`).join('')}}`;
    // additionally, look for index files
    const indexFilenames = extraExtensions.map((ext) => `index.${ext}`);
    // remove the extension from the route
    const reTrim = extensionTest(extraExtensions);

    const updated = { ...routes };

    const docDirs = normalizedDocDirs(dirs);

    docDirs.forEach(({ type, dir }) => {
      const base = path.join(cwd, dir);

      const sourceFiles = globbySync(pattern, { cwd: base });

      sourceFiles.forEach((file) => {
        let endpoint: string;
        if (indexFilenames.includes(path.basename(file))) {
          // /path/to/doc/index.extension => /path/to/doc
          endpoint = path.dirname(file);
        } else {
          // /path/to/doc.extension => /path/to/doc
          endpoint = file.replace(reTrim, '');
        }
        let prefix: string;
        if (!type) {
          // default to /
          prefix = '';
        } else {
          // match Dumi's behavior
          prefix = pluralize.plural(type);
        }
        // remove trailing slashes
        endpoint = path
          .join(prefix, endpoint)
          // remove leading and trailing slashes
          .replace(/^\/+|\/+$/, '');
        if (endpoint === '.') {
          // /index.extension => /
          endpoint = '';
        }
        const route: IRoute = {
          id: endpoint,
          path: endpoint,
          absPath: `/${endpoint}`,
          file: path.resolve(base, file),
          parentId: 'DocLayout',
          // for now this is guaranteed to be called DocLayout
        };
        updated[endpoint] = route;
      });
    });

    return updated;
  };
}
