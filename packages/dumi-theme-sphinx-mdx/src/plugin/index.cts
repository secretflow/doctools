import type { IApi as DumiAPI } from 'dumi';

export default async function shim(api: DumiAPI) {
  const { plugin } = await import('./index.mjs');
  return plugin(api);
}
