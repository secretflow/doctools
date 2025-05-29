import type { Root } from "hast";

export type IntoWorker = {
  code: string;
  lang: string;
};

export type FromWorker = {
  code: string;
  lang: string;
  root: Root;
};

export const colorReplacements = {
  "#fff": "rgb(250, 250, 250)",
} as const;
