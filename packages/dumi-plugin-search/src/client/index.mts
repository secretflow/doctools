import { useEffect, useState } from 'react';

import type { SearchResultList } from '../shared/typing.mjs';
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
      console.error(data.data);
      break;
    case 'ready':
      databaseReady = true;
      break;
    default:
      break;
  }
});

export function useFullTextSearch(query = '') {
  const [ready, setReady] = useState(databaseReady);

  const [results, setResults] = useState<SearchResultList | undefined>(undefined);

  useEffect(() => {
    const listener = ({ data }: MessageEvent<OutgoingMessages>) => {
      switch (data.type) {
        case 'ready':
          databaseReady = true;
          setReady(true);
          break;
        case 'result':
          setResults(data.data);
          break;
        default:
          break;
      }
    };
    worker.addEventListener('message', listener);
    return () => worker.removeEventListener('message', listener);
  }, []);

  useEffect(() => {
    setResults(undefined);
    if (!query) {
      return;
    }
    worker.postMessage(startSearching(query));
  }, [query]);

  return { ready, results, searching: results === undefined };
}
