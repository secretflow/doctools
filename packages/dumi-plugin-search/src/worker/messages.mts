import type { SearchQuery, SearchResultList } from '../shared/typing.mjs';

export type DatabaseRequested = {
  type: 'ready';
};

export type DatabaseReady = {
  type: 'ready';
  /** -1 = failure, 0 ~ 1 = load progress */
  loaded: Record<string, -1 | number>;
};

export type SearchRequested = {
  type: 'search';
  data: SearchQuery;
};

export type SearchResult = {
  type: 'result';
  data: SearchResultList;
};

export type WorkerError = {
  type: 'error';
  data: string;
};

export type IncomingMessages = DatabaseRequested | SearchRequested;

export type OutgoingMessages = DatabaseReady | SearchResult | WorkerError;

export function startSearching(query: SearchQuery): SearchRequested {
  return { type: 'search', data: query };
}

export function requestReady(): DatabaseRequested {
  return { type: 'ready' };
}
