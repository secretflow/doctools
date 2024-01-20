use once_cell::sync::Lazy;
use swc_core::ecma::{
  parser::{Syntax, TsConfig},
  transforms::testing::test,
};

use esm_serve::{externalize_modules, ExternalPackages};

static PACKAGES_ESM_SH: Lazy<ExternalPackages> = Lazy::new(|| {
  ExternalPackages::new()
    .import_from("https://esm.sh/{{package}}@{{version}}{{path}}")
    .package("react", "18.2.0")
    .package("react-dom", "18.2.0")
    .package("lodash", "4.17.21")
    .package("vite", "^5")
});

test!(
  Syntax::Typescript(TsConfig {
    tsx: true,
    decorators: true,
    ..Default::default()
  }),
  |_| externalize_modules(&PACKAGES_ESM_SH),
  everything,
  r#"
    import "vite/client";

    import React from "react";
    import { jsx, jsxs, Fragment } from "react/jsx-runtime";

    import * as _ from "lodash";
    import { unified } from "unified";

    import pkg1 from "react/package.json" with { type: "json" };
    import pkg2 from "react/package.json" assert { type: "json" };

    import { App } from "./App.jsx";

    async function main() {
        const { renderToStaticMarkup } = await import("react-dom/server");
        const { devDependencies } = await import("react/package.json", { with: { type: "json" } });
        return renderToStaticMarkup(<App deps={devDependencies} />);
    }

    export { main, pkg1 };
    export default main;

    export { jsx, jsxs } from "react/jsx-runtime";
    export * from "react";
    export * as mdx from "@mdx-js/mdx";

    @wrapped export class Calculator {}

    export { App } from "./App.jsx";
    "#
);
