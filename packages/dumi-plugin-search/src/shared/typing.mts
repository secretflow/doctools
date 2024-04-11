export type SearchableContent = {
  url: string;
  title: string;
  content: string;
  type: 'page' | 'fragment' | 'symbol';
  project: string;
  version?: string;
  lang?: string;
  symbol?: {
    domain: string;
    name: string;
    module: string;
  };
};

export type SearchResult = {
  id: string;
  document: SearchableContent;
  score?: number;
};

export type SearchResultList = {
  items: SearchResult[];
  totalCount: number;
  elapsedTimeMS?: number;
  queryTokens?: string[];
};

export type SearchQuery = {
  project: string;
  query: string;
  version?: string;
  lang?: string;
  limit?: number;
  offset?: number;
};

export interface SearchProvider {
  load: (data: unknown) => Promise<void>;
  dump: () => Promise<unknown>;
  insert: (...content: SearchableContent[]) => Promise<void>;
  search: (options: SearchQuery) => Promise<SearchResultList>;
}

export type SearchProviderFactory = () => Promise<SearchProvider>;

export type SearchBackendModule = {
  createProvider: SearchProviderFactory;
};
