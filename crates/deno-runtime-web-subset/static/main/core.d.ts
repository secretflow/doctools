/* eslint-disable @typescript-eslint/no-explicit-any */

// https://github.com/denoland/deno_core/blob/0.245.0/core/lib.deno_core.d.ts
// would be cool if it's available through DefinitelyTyped or something

/**
 * @see https://github.com/denoland/deno_core/blob/8cdd2960d37f239f21761c534368924c8a6240c9/core/mod.js#L4
 */
declare module "ext:core/mod.js" {
  export const core: {
    ops: {
      op_snapshot_versions: () => {
        // deno: string;
        v8: string;
        /**
         * https://doc.rust-lang.org/nightly/rustc/platform-support.html
         */
        target: string;
      };
    };
    setBuildInfo: (target: string) => void;
  } & Record<string, any>;
}
