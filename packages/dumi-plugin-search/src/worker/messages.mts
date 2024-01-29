import type { SearchQuery, SearchResultList } from '../shared/typing.mjs';

export type DatabaseReady = {
  type: 'ready';
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

export type IncomingMessages = SearchRequested;

export type OutgoingMessages = DatabaseReady | SearchResult | WorkerError;

export function startSearching(query: SearchQuery): SearchRequested {
  return { type: 'search', data: query };
}
