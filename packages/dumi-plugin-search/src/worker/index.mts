import { createProvider } from 'dumi-plugin-search/runtime/backend';

import type { SearchProvider } from '../shared/typing.mjs';
import { OTHER_PROJECTS } from '../shared/utils/constants.mjs';
import { Future } from '../shared/utils/future.mjs';

import type {
  IncomingMessages,
  WorkerError,
  DatabaseReady,
  SearchRequested,
  SearchResult,
} from './messages.mjs';

import $interconnection$secretpad$spec$trustedflow$psi$other from '?dumi-plugin-search/runtime/interconnection~secretpad~spec~trustedflow~psi~OTHER_PROJECTS';
import $scql$kuscia from '?dumi-plugin-search/runtime/scql~kuscia';
import $secretflow from '?dumi-plugin-search/runtime/secretflow';
import $spu$heu from '?dumi-plugin-search/runtime/spu~heu';

import '../shared/imports.d.mjs';

function onError(e: unknown) {
  self.postMessage({ type: 'error', data: String(e) } satisfies WorkerError);
}

/** secretflow */
let deferredProvider1: Future<SearchProvider> | undefined = undefined;

/** spu, heu */
let deferredProvider2: Future<SearchProvider> | undefined = undefined;

/** scql, kuscia */
let deferredProvider3: Future<SearchProvider> | undefined = undefined;

/** interconnection, secretpad, spec, trustedflow, psi, OTHER */
let deferredProvider4: Future<SearchProvider> | undefined = undefined;

const loaded: Record<string, -1 | number> = {};

function notifyLoaded() {
  self.postMessage({ type: 'ready', loaded } satisfies DatabaseReady);
}

async function loadIndex(url: string, projects: string[]): Promise<SearchProvider> {
  const response = await fetch(url, { mode: 'same-origin' });
  if (!response.ok || response.body === null) {
    projects.forEach((project) => {
      loaded[project] = -1;
    });
    notifyLoaded();
    throw new Error(`Failed to load index from ${url}`);
  }
  const length = Number(response.headers.get('Content-Length'));
  const chunks: Uint8Array[] = [];
  const reader = response.body.getReader();
  let receivedLength = 0;
  let receivedLengthLast = 0;
  // eslint-disable-next-line no-constant-condition
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    if (value === undefined) {
      continue;
    }
    chunks.push(value);
    receivedLength += value.length;
    if (length !== 0 && !isNaN(length)) {
      projects.forEach((project) => {
        loaded[project] = receivedLength / length;
      });
    } else {
      projects.forEach((project) => {
        loaded[project] = 0;
      });
    }
    if (receivedLength - receivedLengthLast >= 4096) {
      receivedLengthLast = receivedLength;
      notifyLoaded();
    }
  }
  const blob = new Blob(chunks);
  try {
    const provider = await createProvider();
    await provider.load(JSON.parse(await blob.text()));
    projects.forEach((project) => {
      loaded[project] = 1;
    });
    return provider;
  } catch (e) {
    projects.forEach((project) => {
      loaded[project] = -1;
    });
    notifyLoaded();
    throw e;
  }
}

async function initSearch1() {
  deferredProvider1 = new Future();
  deferredProvider1.fulfill(await loadIndex($secretflow, ['secretflow']));
  notifyLoaded();
}

async function initSearch2() {
  deferredProvider2 = new Future();
  deferredProvider2.fulfill(await loadIndex($spu$heu, ['spu', 'heu']));
  notifyLoaded();
}

async function initSearch3() {
  deferredProvider3 = new Future();
  deferredProvider3.fulfill(await loadIndex($scql$kuscia, ['scql', 'kuscia']));
  notifyLoaded();
}

async function initSearch4() {
  deferredProvider4 = new Future();
  deferredProvider4.fulfill(
    await loadIndex($interconnection$secretpad$spec$trustedflow$psi$other, [
      'interconnection',
      'secretpad',
      'spec',
      'trustedflow',
      'psi',
      OTHER_PROJECTS,
    ]),
  );
  notifyLoaded();
}

async function startSearching(event: SearchRequested) {
  const deferredProvider = (() => {
    switch (event.data.project) {
      case 'secretflow':
        return deferredProvider1;
      case 'spu':
      case 'heu':
        return deferredProvider2;
      case 'scql':
      case 'kuscia':
        return deferredProvider3;
      default:
        return deferredProvider4;
    }
  })();
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
    case 'ready':
      notifyLoaded();
      break;
    default:
      return;
  }
});

initSearch1().catch(onError);
initSearch2().catch(onError);
initSearch3().catch(onError);
initSearch4().catch(onError);
