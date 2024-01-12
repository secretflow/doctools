import "https://esm.sh/vite@^5/client";
import React from "https://esm.sh/react@18.2.0";
import { jsx, jsxs, Fragment } from "https://esm.sh/react@18.2.0/jsx-runtime";
import * as _ from "https://esm.sh/lodash@4.17.21";
import { unified } from "https://esm.sh/unified@%3E=0.0.0";
import pkg1 from "https://esm.sh/react@18.2.0/package.json" with {
    type: "json"
};
import pkg2 from "https://esm.sh/react@18.2.0/package.json" with {
    type: "json"
};
import { App } from "./App.jsx";
async function main() {
    const { renderToStaticMarkup } = await import("https://esm.sh/react-dom@18.2.0/server");
    const { devDependencies } = await import("https://esm.sh/react@18.2.0/package.json", {
        with: {
            type: "json"
        }
    });
    return renderToStaticMarkup(<App deps={devDependencies}/>);
}
export { main, pkg1 };
export default main;
export { jsx, jsxs } from "https://esm.sh/react@18.2.0/jsx-runtime";
export * from "https://esm.sh/react@18.2.0";
export * as mdx from "https://esm.sh/@mdx-js/mdx@%3E=0.0.0";
@wrapped
export class Calculator {
}
export { App } from "./App.jsx";
