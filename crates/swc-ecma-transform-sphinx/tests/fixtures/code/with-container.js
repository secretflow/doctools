/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-undef */

import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";

export const App = () =>
  _jsx(container, {
    ids: [],
    classes: ["outer-wrapper"],
    names: [],
    dupnames: [],
    children: _jsxs(container, {
      ids: ["id1"],
      classes: ["literal-block-wrapper"],
      names: [],
      dupnames: [],
      literal_block: true,
      children: [
        _jsx(caption, {
          ids: [],
          classes: [],
          names: [],
          dupnames: [],
          children: "typescript.ts",
        }),
        _jsx(literal_block, {
          ids: [],
          classes: [],
          names: [],
          dupnames: [],
          language: "typescript",
          children: `import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";`,
        }),
      ],
    }),
  });
