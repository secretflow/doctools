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
