import { highlight } from "./server";

self.addEventListener("message", async (event) => {
  self.postMessage(await highlight(event.data));
});
