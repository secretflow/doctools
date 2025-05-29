import type { SearchQuery, SearchResultList } from "../../search/index.ts";

export type DatabaseReady = {
  type: "ready";
  triple: string;
};

export type LoadRequested = {
  type: "load";
  triple: string;
  data: { contentType: string; buffer: Uint8Array };
};

export type SearchRequested = {
  type: "search";
  triple: string;
  req: SearchQuery;
};

export type SearchResult = {
  type: "result";
  triple: string;
  req: SearchQuery;
  res: SearchResultList;
};

export type WorkerError = {
  type: "error";
  data: string;
};

export type IntoWorker = LoadRequested | SearchRequested;

export type FromWorker = DatabaseReady | SearchResult | WorkerError;
