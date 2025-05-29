import type { Orama } from "@orama/orama";
import { create, search } from "@orama/orama";

type DataType<T> = T extends string
  ? "string"
  : T extends number
    ? "number"
    : T extends boolean
      ? "boolean"
      : T extends (infer U)[]
        ? U extends string | number | boolean
          ? `${DataType<U>}[]`
          : never
        : T extends Partial<Record<infer K, unknown>>
          ? Partial<{ [P in K]: DataType<T[P]> }>
          : never;

export type SearchableContent = {
  url: string;
  title: string;
  content: string;
  type: "page" | "fragment" | "symbol";
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
  query: string;
  limit?: number | undefined;
  offset?: number | undefined;
};

const schema: DataType<SearchableContent> = {
  url: "string",
  title: "string",
  content: "string",
  type: "string",
  symbol: {
    domain: "string",
    name: "string",
    module: "string",
  },
};

const tokenize = (raw: string) =>
  raw
    // remove all non-word characters
    .replaceAll(/[^_\d\p{Script=Han}\p{Script=Latin}]+/gu, " ")
    // insert spaces between every Chinese characters
    .replaceAll(/([\p{Script=Han}])/gu, " $1 ")
    // split by spaces
    .split(" ")
    // remove empty strings
    .filter((x) => x)
    .map((x) => x.toLocaleLowerCase());

export async function createDatabase(): Promise<Orama<typeof schema>> {
  return await create({
    schema,
    components: {
      tokenizer: {
        tokenize,
        language: "english",
        normalizationCache: new Map(),
      },
    },
  });
}

export async function searchDatabase(
  db: Orama<typeof schema>,
  { query, limit = 10, offset = 0 }: SearchQuery,
): Promise<SearchResultList> {
  const result = await search(db, {
    term: query,
    limit,
    offset,
    where: { type: "fragment" },
  });
  return {
    totalCount: result.count,
    items: result.hits as SearchResult[],
    elapsedTimeMS: result.elapsed.raw,
    queryTokens: tokenize(query),
  };
}
