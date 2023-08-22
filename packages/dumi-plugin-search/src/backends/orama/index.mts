import * as orama from '@orama/orama';

import type {
  SearchBackendModule,
  SearchableContent,
  SearchResult,
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
  topic: 'string',
  lang: 'string',
  symbol: {
    domain: 'string',
    name: 'string',
    module: 'string',
  },
};

export const createProvider: SearchBackendModule['createProvider'] = async function () {
  const db = await orama.create({
    schema,
    components: {
      tokenizer: {
        tokenize: (raw) =>
          raw
            .replaceAll(/[^.\d\p{Script=Han}\p{Script=Latin}]+/gu, ' ')
            .split(' ')
            .filter((x) => x),
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

  async function search(query: string) {
    const result = await orama.search(db, {
      term: query,
      limit: 30,
      threshold: 0.5,
    });
    return { totalCount: result.count, items: result.hits as SearchResult[] };
  }

  return { load, dump, insert, search };
};
