import type { Plugin } from 'unified';

import type { SearchBackendModule, SearchableContent } from '../shared/typing.mjs';

import type { LoaderConfig } from './typing.d.js';

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

export async function loader({ backend, pipelines = {}, routes }: LoaderConfig) {
  const unistCompilerNoOp: Plugin = function () {
    this.Compiler = (_tree, file) => file;
  };

  const { unified } = await import('unified');
  const { rehypeArticleOutline } = await import(
    '@secretflow/unified-toolkit/rehype-article-outline'
  );

  const PROCESSORS = Object.fromEntries(
    Object.entries(pipelines).map(([extension, factory]) => {
      let processor = factory(unified());
      processor = processor.use(rehypeArticleOutline).use(unistCompilerNoOp);
      return [extension, processor.freeze()];
    }),
  );

  const { read } = await import('to-vfile');

  const { createProvider } = (await import(backend)) as SearchBackendModule;
  const database = await createProvider();

  const endpoints = routes ?? {};

  for (const chunk of chunked(Object.values(endpoints), 64)) {
    await Promise.all(
      chunk.map(async ({ absPath, file }) => {
        const extension = file?.split('.').pop();

        if (!file || !extension) {
          return;
        }

        const processor = PROCESSORS[extension];

        if (!processor) {
          return;
        }

        const source = await read(file);
        const result = await processor.process(source);
        const targets: SearchableContent[] = [];

        result.data.outline?.forEach(({ id, longTitle, content }) => {
          const url = id ? `${absPath}#${id}` : absPath;
          targets.push({
            url,
            title: longTitle,
            content,
            type: 'prose',
          });
        });

        await database.insert(...targets);
      }),
    );
  }

  const serialized = await database.dump();
  return JSON.stringify(serialized);
}
