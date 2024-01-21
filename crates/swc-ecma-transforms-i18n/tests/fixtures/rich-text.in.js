/* eslint-disable */

jsxs("p", {
  children: [
    "The quick ",
    jsx("em", { children: "brown" }),
    " fox ",
    jsx("strong", { children: "jumps" }),
    " over the ",
    jsxs("a", {
      href: "https://en.wikipedia.org/wiki/The_quick_brown_fox_jumps_over_the_lazy_dog",
      children: ["lazy ", jsx("span", { children: "dog" })],
    }),
    ".",
  ],
});
