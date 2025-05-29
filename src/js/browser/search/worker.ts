import { load } from "@orama/orama";
import { gunzipSync } from "fflate";

import { createDatabase, searchDatabase } from "../../search";

import type {
  DatabaseReady,
  IntoWorker,
  SearchRequested,
  SearchResult,
  WorkerError,
} from "./types";

const providers: Map<string, Database> = new Map();

type Database = ReturnType<typeof createDatabase>;

function onError(e: unknown) {
  self.postMessage({ type: "error", data: String(e) } satisfies WorkerError);
  console.error(e);
}

async function startSearching({ triple, req }: SearchRequested) {
  const deferredProvider = (() => {
    const provider = providers.get(triple);
    if (!provider) {
      throw new Error("unreachable");
    } else {
      return provider;
    }
  })();
  if (deferredProvider === undefined) {
    return;
  }
  const provider = await deferredProvider;
  const res = await searchDatabase(provider, req);
  self.postMessage({ type: "result", triple, req, res } satisfies SearchResult);
}

self.addEventListener("message", ({ data }: MessageEvent<IntoWorker>) => {
  switch (data.type) {
    case "search":
      startSearching(data).catch(onError);
      break;
    case "load":
      {
        const text = (() => {
          if (data.data.contentType.startsWith("application/json")) {
            return new TextDecoder().decode(data.data.buffer);
          } else {
            const decompressed = gunzipSync(data.data.buffer);
            return new TextDecoder().decode(decompressed);
          }
        })();
        const serialized = JSON.parse(text);
        const database = (async () => {
          const database = await createDatabase();
          await load(database, serialized);
          return database;
        })();
        providers.set(data.triple, database);
        self.postMessage({
          type: "ready",
          triple: data.triple,
        } satisfies DatabaseReady);
      }
      break;
  }
});
