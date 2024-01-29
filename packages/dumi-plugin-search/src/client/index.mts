import { useEffect, useRef, useState } from 'react';

import type { SearchQuery, SearchResultList } from '../shared/typing.mjs';
import { startSearching } from '../worker/messages.mjs';
import type { OutgoingMessages } from '../worker/messages.mjs';

import '../shared/imports.d.mjs';

let databaseReady = false;

const worker = new Worker(
  new URL('dumi-plugin-search/runtime/worker', import.meta.url),
);

worker.addEventListener('message', ({ data }: MessageEvent<OutgoingMessages>) => {
  switch (data.type) {
    case 'error':
      console.error('Error in worker', data.data);
      break;
    case 'ready':
      databaseReady = true;
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
  offset,
}: SearchQuery) {
  const [ready, setReady] = useState(databaseReady);

  const [results, setResults] = useState<SearchResultList | undefined>(undefined);

  const currentOffset = useRef(offset);
  currentOffset.current = offset;

  useEffect(() => {
    const listener = ({ data }: MessageEvent<OutgoingMessages>) => {
      switch (data.type) {
        case 'ready':
          databaseReady = true;
          setReady(true);
          break;
        case 'result':
          if (currentOffset.current !== 0) {
            setResults((prev) => {
              if (prev === undefined) {
                return prev;
              }
              return {
                ...prev,
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
    // setResults(undefined);
    if (!query) {
      return;
    }
    worker.postMessage(startSearching({ project, query, version, lang, offset }));
  }, [project, query, offset, version, lang]);

  return { ready, results, searching: results === undefined };
}
