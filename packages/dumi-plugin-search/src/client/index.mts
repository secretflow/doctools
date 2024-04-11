import { useEffect, useRef, useState, useSyncExternalStore } from 'react';

import type { SearchQuery, SearchResultList } from '../shared/typing.mjs';
import { requestReady, startSearching } from '../worker/messages.mjs';
import type { OutgoingMessages } from '../worker/messages.mjs';

import '../shared/imports.d.mjs';

let loaded: Record<string, number> = {};

const worker = new Worker(
  new URL('dumi-plugin-search/runtime/worker', import.meta.url),
);

worker.addEventListener('message', ({ data }: MessageEvent<OutgoingMessages>) => {
  switch (data.type) {
    case 'error':
      console.error('Error in worker', data.data);
      break;
    case 'ready':
      loaded = data.loaded;
      break;
    default:
      break;
  }
});

export function useFullTextSearch({
  project,
  version,
  lang,
  query,
  limit,
  offset,
}: SearchQuery) {
  const [results, setResults] = useState<SearchResultList | undefined>(undefined);

  const currentOffset = useRef(offset);
  currentOffset.current = offset;

  useSyncExternalStore(
    (changed) => {
      const listener = ({ data }: MessageEvent<OutgoingMessages>) => {
        switch (data.type) {
          case 'ready':
            loaded = data.loaded;
            changed();
            break;
          default:
            break;
        }
      };
      worker.addEventListener('message', listener);
      return () => worker.removeEventListener('message', listener);
    },
    () => loaded,
  );

  useEffect(() => {
    const listener = ({ data }: MessageEvent<OutgoingMessages>) => {
      switch (data.type) {
        case 'result':
          if (currentOffset.current !== 0) {
            setResults((prev) => {
              if (prev === undefined) {
                return prev;
              }
              return {
                ...data.data,
                items: [...prev.items, ...data.data.items],
              };
            });
          } else {
            setResults(data.data);
          }
          break;
        default:
          break;
      }
    };
    worker.addEventListener('message', listener);
    return () => worker.removeEventListener('message', listener);
  }, []);

  useEffect(() => {
    if (!query) {
      return;
    }
    worker.postMessage(
      startSearching({ project, query, version, lang, limit, offset }),
    );
  }, [project, query, offset, version, lang, limit]);

  useEffect(() => {
    worker.postMessage(requestReady());
  }, []);

  return {
    loaded,
    ready: loaded[project] === 1,
    results,
    searching: results === undefined,
  };
}

export { OTHER_PROJECTS } from '../shared/utils/constants.mjs';
