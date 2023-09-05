import type { IApi as DumiAPI } from 'dumi';

/** Create a unique key for a loader, plugin, etc */
export const uniqueKey = (name: string, ...ids: string[]) =>
  `dumi:${name}:${ids.join('~')}`;

/**
 * Create a regex that test if a filename has one of the given extensions
 *
 * @param extensions Extensions without the leading dot
 * @returns
 */
export const extensionTest = (extensions: string[]) =>
  new RegExp(`\\.(${extensions.join('|')})$`);

export type DocDirConfig = DumiAPI['config']['resolve']['docDirs'][number];

type NormalizedDocDirConfig = Exclude<DocDirConfig, string>;

// from Dumi
export function normalizedDocDirs(docDirs: DocDirConfig[]): NormalizedDocDirConfig[] {
  return docDirs.map((dir) => (typeof dir === 'object' ? dir : { dir }));
}
