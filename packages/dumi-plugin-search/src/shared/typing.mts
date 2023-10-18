export type SearchableContent = {
  url: string;
  title: string;
  content: string;
  type: 'prose' | 'symbol';
  topic?: string;
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

export interface SearchProvider {
  load: (data: unknown) => Promise<void>;
  dump: () => Promise<unknown>;
  insert: (...content: SearchableContent[]) => Promise<void>;
  search: (query: string) => Promise<SearchResultList>;
}

export type SearchProviderFactory = () => Promise<SearchProvider>;

export type SearchBackendModule = {
  createProvider: SearchProviderFactory;
};
