declare module 'dumi-plugin-search/runtime/worker';

declare module 'dumi-plugin-search/runtime/backend' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-imports
  const exports: import('./typing.mjs').SearchBackendModule;
  export = exports;
}

declare module '?dumi-plugin-search/runtime/index' {
  const data: unknown;
  export = data;
}
