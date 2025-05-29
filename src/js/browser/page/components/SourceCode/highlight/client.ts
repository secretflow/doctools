import type { IntoWorker, FromWorker } from "../types";

// eslint-disable-next-line import/default
import Worker from "./worker?worker&inline";

const worker = new Worker();

export function highlight({ code, lang }: IntoWorker) {
  return new Promise<FromWorker>((resolve) => {
    function listener(event: MessageEvent<FromWorker>) {
      const { code: id, root, lang } = event.data;
      if (id !== code) {
        return;
      }
      worker.removeEventListener("message", listener);
      resolve({ root, code, lang });
    }
    worker.addEventListener("message", listener);
    worker.postMessage({ code, lang } satisfies IntoWorker);
  });
}
