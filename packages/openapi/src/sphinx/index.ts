import { createElement } from "react";
import { createRoot } from "react-dom/client";

declare global {
  interface Window {
    define: unknown;
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  const sites: [Element, string][] = [];

  document.querySelectorAll("div.highlight-swagger").forEach((elem) => {
    const raw = elem.querySelector("pre")?.textContent;
    if (!raw) {
      return;
    }
    sites.push([elem, raw]);
    createRoot(elem).render(createElement("div", null, "Loading..."));
  });

  await (async () => {
    // https://github.com/no-context/moo/blob/main/moo.js#L1-L9
    const define = window.define;
    window.define = undefined;
    await import("@lingui/core");
    window.define = define;
  })();

  const { App } = await import("./App");

  sites.forEach(([elem, schema]) => {
    createRoot(elem).render(createElement(App, { schema }));
  });
});
