import { createRequire } from "node:module";
import { setImmediate } from "node:timers";

const global = globalThis;

const self = typeof globalThis.self === "undefined" ? globalThis : globalThis.self;

const require =
  typeof globalThis.require === "undefined"
    ? createRequire(import.meta.url)
    : globalThis.require;

Object.entries({
  global,
  require,
  self,
  setImmediate,
}).forEach(([k, v]) =>
  Object.defineProperty(globalThis, k, {
    get: () => v,
    configurable: false,
    enumerable: false,
  }),
);
