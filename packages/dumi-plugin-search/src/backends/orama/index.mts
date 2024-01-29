import * as orama from '@orama/orama';
import type { WhereCondition } from '@orama/orama';

import type {
  SearchBackendModule,
  SearchableContent,
  SearchResult,
  SearchResultList,
  SearchQuery,
} from '../../shared/typing.mjs';

type DataType<T> = T extends string
  ? 'string'
  : T extends number
  ? 'number'
  : T extends boolean
  ? 'boolean'
  : // : T extends (infer U)[]
  // ? U extends string | number | boolean
  //   ? `${DataType<U>}[]`
  //   : never
  T extends Partial<Record<infer K, unknown>>
  ? Partial<{ [P in K]: DataType<T[P]> }>
  : never;

const schema: DataType<SearchableContent> = {
  url: 'string',
  title: 'string',
  content: 'string',
  type: 'string',
  project: 'string',
  version: 'string',
  lang: 'string',
  symbol: {
    domain: 'string',
    name: 'string',
    module: 'string',
  },
};

const tokenize = (raw: string) =>
  raw
    // remove all non-word characters
    .replaceAll(/[^_\d\p{Script=Han}\p{Script=Latin}]+/gu, ' ')
    // insert spaces between every Chinese characters
    .replaceAll(/([\p{Script=Han}])/gu, ' $1 ')
    // split by spaces
    .split(' ')
    // remove empty strings
    .filter((x) => x)
    .map((x) => x.toLocaleLowerCase());

export const createProvider: SearchBackendModule['createProvider'] = async function () {
  const db = await orama.create({
    schema,
    components: {
      tokenizer: {
        tokenize,
        language: 'english',
        normalizationCache: new Map(),
      },
    },
  });

  async function load(data: unknown) {
    await orama.load(db, data as orama.RawData);
  }

  async function dump() {
    return await orama.save(db);
  }

  async function insert(...content: SearchableContent[]) {
    await orama.insertMultiple(db, content);
  }

  async function search({
    project,
    query,
    version,
    lang,
    limit = 10,
    offset = 0,
  }: SearchQuery): Promise<SearchResultList> {
    // TODO: upstream typing issues
    const where: WhereCondition<
      Partial<{ project: 'string'; version: 'string'; lang: 'string' }>
    > = {};
    if (project) {
      where.project = project;
    }
    if (version) {
      where.version = version;
    }
    if (lang) {
      where.lang = lang;
    }
    const result = await orama.search(db, {
      term: query,
      limit,
      offset,
      where,
    });
    return {
      totalCount: result.count,
      items: result.hits as SearchResult[],
      elapsedTimeMS: result.elapsed.raw,
      queryTokens: tokenize(query),
    };
  }

  return { load, dump, insert, search };
};
