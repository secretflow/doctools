import type { SearchProvider } from '../shared/typing.mjs';
import { Future } from '../shared/utils/future.mjs';

import type {
  IncomingMessages,
  WorkerError,
  DatabaseReady,
  SearchRequested,
  SearchResult,
} from './messages.mjs';

import '../shared/imports.d.mjs';

let deferredProvider: Future<SearchProvider> | undefined = undefined;

function onError(e: unknown) {
  self.postMessage({ type: 'error', data: String(e) } satisfies WorkerError);
}

async function initSearch() {
  deferredProvider = new Future();
  const [{ createProvider }, { default: data }] = await Promise.all([
    import('dumi-plugin-search/runtime/backend'),
    import('?dumi-plugin-search/runtime/index'),
  ]);
  const provider = await createProvider();
  await provider.load(data);
  deferredProvider.fulfill(provider);
  self.postMessage({ type: 'ready' } satisfies DatabaseReady);
}

async function startSearching(event: SearchRequested) {
  if (deferredProvider === undefined) {
    return;
  }
  const provider = await deferredProvider;
  const results = await provider.search(event.data);
  self.postMessage({ type: 'result', data: results } satisfies SearchResult);
}

self.addEventListener('message', ({ data }: MessageEvent<IncomingMessages>) => {
  switch (data.type) {
    case 'search':
      startSearching(data).catch(onError);
      break;
    default:
      return;
  }
});

initSearch().catch(onError);
