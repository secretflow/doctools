/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-undef */

import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";

export default function App() {
  return _jsxs("section", {
    children: [
      _jsxs(paragraph, {
        classes: [],
        dupnames: [],
        ids: ["para1"],
        names: [],
        children: [
          "Lorem ",
          _jsx("strong", {
            children: "ipsum",
          }),
          " ",
          _jsx(raw, {
            classes: [],
            dupnames: [],
            ids: ["warning"],
            names: [],
            format: "html",
            children: `<span style="color: red;">`,
          }),
          _jsx("em", {
            children: "dolor",
          }),
          " sit",
          _jsx(raw, {
            classes: [],
            dupnames: [],
            ids: [],
            names: [],
            format: "html",
            children: `</span>`,
          }),
          " amet.",
        ],
      }),
      _jsx(raw, {
        classes: [],
        dupnames: [],
        ids: [],
        names: [],
        format: "html",
        children: `<hr>`,
      }),
      _jsx(paragraph, {
        classes: [],
        dupnames: [],
        ids: ["para2"],
        names: [],
        children: "Consectetur adipiscing elit.",
      }),
    ],
  });
}
