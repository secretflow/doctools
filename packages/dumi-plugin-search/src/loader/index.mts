import type { Plugin } from 'unified';

import type { SearchBackendModule, SearchableContent } from '../shared/typing.mjs';
import { OTHER_PROJECTS } from '../shared/utils/constants.mjs';

import type { LoaderConfig } from './typing.d.js';

const RE_STANDARD_PROJECT_PATH =
  /^\/docs\/(?<project>[^/]+)\/(?<version>[^/]+)\/(?<lang>[^/]+)/i;

function* chunked<T>(items: Iterable<T>, size: number): Generator<T[], void, void> {
  const iterator = items[Symbol.iterator]();
  let result = iterator.next();
  while (!result.done) {
    const chunk: T[] = [];
    for (let i = 0; i < size && !result.done; i++) {
      chunk.push(result.value);
      result = iterator.next();
    }
    yield chunk;
  }
}

export async function loader(
  query: string,
  { backend, pipelines = {}, routes }: LoaderConfig,
) {
  const unistCompilerNoOp: Plugin = function () {
    this.Compiler = (_tree, file) => file;
  };

  const { unified } = await import('unified');
  const { rehypeArticleOutline } = await import(
    '@secretflow/unified-toolkit/rehype-article-outline'
  );

  const PROCESSORS = Object.fromEntries(
    Object.entries(pipelines).map(
      ([extension, { preprocessor, processor: factory }]) => {
        let processor = factory(unified());
        processor = processor.use(rehypeArticleOutline).use(unistCompilerNoOp);
        return [extension, { preprocessor, processor: processor.freeze() }];
      },
    ),
  );

  const { read } = await import('to-vfile');

  const { createProvider } = (await import(backend)) as SearchBackendModule;
  const database = await createProvider();

  const endpoints = routes ?? {};

  const shards = query.split('/').pop()?.split('~') ?? [];

  for (const chunk of chunked(Object.values(endpoints), 64)) {
    await Promise.all(
      chunk.map(async ({ absPath, file }) => {
        const extension = file?.split('.').pop();

        const pathMatch = RE_STANDARD_PROJECT_PATH.exec(absPath);

        if (!file || !extension) {
          return;
        }

        const { preprocessor, processor } = PROCESSORS[extension] || {};

        if (!processor) {
          return;
        }

        if (
          pathMatch &&
          !shards.includes(pathMatch.groups?.['project'] ?? OTHER_PROJECTS)
        ) {
          return;
        }

        const source = await read(file);

        if (preprocessor) {
          source.value = Buffer.from(
            preprocessor(source.value.toString('utf-8')),
            'utf-8',
          );
        }

        const result = await processor.process(source);
        const targets: SearchableContent[] = [];

        result.data.outline?.forEach(({ id, longTitle, content }) => {
          const url = id ? `${absPath}#${id}` : absPath;
          targets.push({
            url,
            title: longTitle,
            content,
            project: pathMatch?.groups?.['project'] || OTHER_PROJECTS,
            version: pathMatch?.groups?.['version'],
            lang: pathMatch?.groups?.['lang'],
            type: id ? 'fragment' : 'page',
          });
        });

        await database.insert(...targets);
      }),
    );
  }

  const serialized = await database.dump();
  return JSON.stringify(serialized);
}
