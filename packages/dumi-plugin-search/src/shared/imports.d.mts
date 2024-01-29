declare module 'dumi-plugin-search/runtime/worker';

declare module 'dumi-plugin-search/runtime/backend' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-imports
  const exports: import('./typing.mjs').SearchBackendModule;
  export = exports;
}

declare module '?dumi-plugin-search/runtime/secretflow' {
  const url: string;
  export = url;
}

declare module '?dumi-plugin-search/runtime/spu~heu' {
  const url: string;
  export = url;
}

declare module '?dumi-plugin-search/runtime/scql~kuscia' {
  const url: string;
  export = url;
}

declare module '?dumi-plugin-search/runtime/interconnection~secretpad~spec~trustedflow~psi~OTHER_PROJECTS' {
  const url: string;
  export = url;
}
