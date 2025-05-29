import { keepPreviousData, useInfiniteQuery, useQuery } from "@tanstack/react-query";

import type { SearchQuery, SearchResultList } from "../../search/index";
import { useRepoContent } from "../app";

import type { FromWorker, LoadRequested, SearchRequested } from "./types";
// eslint-disable-next-line import/default
import SearchWorker from "./worker.ts?worker&inline";

const worker = new SearchWorker();

worker?.addEventListener("message", ({ data }: MessageEvent<FromWorker>) => {
  switch (data.type) {
    case "error":
      console.error("Error in worker", data.data);
      break;
  }
});

type DownloadProgress = {
  received: number;
  sizeHint: {
    lower: number;
    upper: number | null;
  };
};

const loaded: { [x: string]: DownloadProgress | undefined } = {};

export function useFullTextSearch({ query, limit = 10 }: Omit<SearchQuery, "offset">) {
  const {
    repo: {
      project: {
        triple,
        module: { database },
      },
    },
  } = useRepoContent();

  const key = String(useFullTextSearch) + String(database);

  const {
    promise: loadDatabase,
    isSuccess: ready,
    error,
  } = useQuery({
    queryFn: async () => {
      const { ok, body, headers } = await database();

      if (!ok || body === null) {
        throw new Error(`Failed to load index from ${triple}`);
      }

      const buffer = await (async () => {
        loaded[key] = undefined;
        const length = Number(headers.get("Content-Length"));
        const chunks: Uint8Array[] = [];
        const reader = body.getReader();
        let received = 0;
        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            break;
          }
          if (value === undefined) {
            continue;
          }
          chunks.push(value);
          loaded[key] = {
            received,
            sizeHint: { lower: length, upper: null },
          };
          received += value.length;
        }
        return new Uint8Array(await new Blob(chunks).arrayBuffer());
      })();

      const contentType = headers.get("Content-Type") || "application/octet-stream";

      await new Promise<void>((resolve) => {
        const listener = ({ data }: MessageEvent<FromWorker>) => {
          switch (data.type) {
            case "ready":
              if (data.triple === triple.join("/")) {
                worker.removeEventListener("message", listener);
                resolve();
              }
          }
        };

        worker?.addEventListener("message", listener);

        worker?.postMessage({
          type: "load",
          triple: triple.join("/"),
          data: { contentType, buffer },
        } satisfies LoadRequested);
      });

      return true;
    },
    queryKey: [key],
    experimental_prefetchInRender: true,
  });

  const {
    data: results,
    isFetching: searching,
    fetchNextPage: next,
  } = useInfiniteQuery({
    queryKey: [triple, query, limit],

    queryFn: async ({ pageParam: offset }) => {
      await loadDatabase;

      return await new Promise<SearchResultList>((resolve) => {
        const listener = ({ data }: MessageEvent<FromWorker>) => {
          switch (data.type) {
            case "result":
              if (
                data.triple === triple.join("/") &&
                data.req.query === query &&
                data.req.limit === limit &&
                data.req.offset === offset
              ) {
                worker.removeEventListener("message", listener);
                resolve(data.res);
              }
          }
        };

        worker.addEventListener("message", listener);

        worker.postMessage({
          type: "search",
          triple: triple.join("/"),
          req: {
            query,
            limit,
            offset,
          },
        } satisfies SearchRequested);
      });
    },

    getNextPageParam: (_, pages) => pages.length * limit,
    initialPageParam: 0,
    placeholderData: keepPreviousData,
  });

  if (error) {
    console.error(error);
    delete loaded[key];
  }

  return {
    results,
    searching,
    next,
    ready,
    error,
    loaded: () => loaded[key],
  };
}
