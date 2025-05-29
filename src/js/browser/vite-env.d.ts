/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_SERVER_PORT: number;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.po" {
  import type { Messages } from "@lingui/core";
  export const messages: Messages;
}
