/* eslint-disable @typescript-eslint/no-unused-vars */ /* eslint-disable no-undef */ import {
  jsx as _jsx,
  jsxs as _jsxs,
} from "react/jsx-runtime";
export default function App() {
  return /*#__PURE__*/ _jsxs(section, {
    ids: ["heading-level-3"],
    names: ["heading level 3"],
    children: [
      /*#__PURE__*/ _jsx(title, {
        children: "Heading Level 3",
      }),
      /*#__PURE__*/ _jsx(paragraph, {
        classes: ["bg-primary"],
        children: "Here is a paragraph with a class to control its formatting.",
      }),
      /*#__PURE__*/ _jsx(transition, {}),
      /*#__PURE__*/ _jsxs(paragraph, {
        children: [
          /*#__PURE__*/ _jsx(strong, {
            children: "strong",
          }),
          ", ",
          /*#__PURE__*/ _jsx(emphasis, {
            children: "emphasis",
          }),
          ", ",
          /*#__PURE__*/ _jsx(literal, {
            children: "literal text",
          }),
          ", *escaped symbols*",
        ],
      }),
    ],
  });
}
