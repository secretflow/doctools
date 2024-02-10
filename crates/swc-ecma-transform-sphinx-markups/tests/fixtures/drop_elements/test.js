import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";

_jsxs(_Fragment, {
  children: [
    _jsxs("div", {
      children: [
        "Lorem ipsum",
        _jsx(_Fragment, {
          children: _jsx("comment", {
            children: "This is a comment",
          }),
        }),
        "dolor sit amet",
      ],
    }),
    _jsx("comment", {
      children: "This is another comment",
    }),
  ],
});
