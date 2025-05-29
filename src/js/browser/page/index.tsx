import { MDXProvider } from "@mdx-js/react";
import type { ComponentProps, PropsWithChildren, ReactNode } from "react";
import { Helmet } from "react-helmet-async";
import type { Link as RouterLink } from "react-router";
import { styled } from "styled-components";

import { breakpoint, theme } from "../theme";

import { Container } from "./components/Container";
import { DefinitionList } from "./components/DefinitionList";
import { FieldList } from "./components/FieldList";
import { Footnote } from "./components/Footnote";
import { Graphviz } from "./components/Graphviz";
import { HorizontalList } from "./components/HorizontalList";
import { ImageViewer } from "./components/ImageViewer";
import { LineBlock } from "./components/LineBlock";
import { InlineMath, Math } from "./components/Math";
import { Mermaid } from "./components/Mermaid";
import { Notebook } from "./components/Notebook";
import { OpenAPI } from "./components/OpenAPI";
import { OptionList } from "./components/OptionList";
import { Outline } from "./components/Outline";
import { SourceCode } from "./components/SourceCode";
import { SphinxDesign } from "./components/SphinxDesign";
import { Table } from "./components/Table";
import { TableOfContents } from "./components/TableOfContents";
import { Target } from "./components/Target";
import { _Line } from "./components/_Line";
import { a, code, h1, h2, h3, h4, h5, h6 } from "./components/intrinsic";

export type PageRendererProps = PropsWithChildren<{
  Link: (props: ComponentProps<typeof RouterLink>) => ReactNode;
}>;

export function PageRenderer({ Link, children }: PageRendererProps) {
  return (
    <MDXProvider
      components={{
        Container,
        DefinitionList,
        FieldList,
        Footnote,
        Graphviz,
        Helmet,
        HorizontalList,
        InlineMath,
        LineBlock,
        Link,
        Math,
        Mermaid,
        Notebook,
        OpenAPIViewer: OpenAPI,
        OptionList,
        Outline,
        SourceCode,
        _Line,
        SphinxDesign,
        Table,
        TableOfContents,
        Target,
        a,
        code,
        h1,
        h2,
        h3,
        h4,
        h5,
        h6,
        img: ImageViewer,
      }}
    >
      <Article>{children}</Article>
    </MDXProvider>
  );
}

const { mobileToolbarHeight, navbarHeight, scrollOffset } = theme.dimensions;

const Article = styled.article`
  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;
  min-width: 0;
  height: 100%;
  font-family: ${theme.fonts.sansSerif};
  font-size: 1rem;
  line-height: 1.65rem;
  color: ${theme.colors.fg.default};

  * {
    box-sizing: border-box;
  }

  > * {
    min-width: 0;
    max-width: 100%;
  }

  [id] {
    /*
      https://issues.chromium.org/issues/40074749
      https://issues.chromium.org/issues/40822817
    */

    scroll-margin-top: calc(${navbarHeight} + ${scrollOffset});

    ${breakpoint("mobileWidth")} {
      scroll-margin-top: calc(
        ${navbarHeight} + ${scrollOffset} + ${mobileToolbarHeight}
      );
    }
  }

  h1 {
    margin: 1.5rem 0 0;
    font-size: 2rem;
    font-weight: 600;
    line-height: 1.5;
  }

  > h1:first-child {
    margin-block-start: 0 !important;
  }

  h2 {
    margin: 1.5rem 0 0;
    font-size: 1.5rem;
    font-weight: 600;
    line-height: 1.5;
  }

  h1 + h2 {
    margin-top: 0;
  }

  h3 {
    margin: 1rem 0 0;
    font-size: 1.3rem;
    font-weight: 600;
    line-height: 1.4;
  }

  h4 {
    margin: 0.5rem 0 0;
    font-size: 1.1rem;
    font-weight: 600;
    line-height: 1.4;
  }

  h5 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }

  h6 {
    margin: 0;
    font-size: 1rem;
    font-style: italic;
    font-weight: 500;
  }

  em {
    font-weight: 500;
  }

  strong {
    font-weight: 600;
  }

  p,
  pre,
  ul,
  ol {
    margin: 0;
  }

  pre,
  code {
    font-family: ${theme.fonts.monospace};
  }

  code {
    color: ${theme.colors.fg.strong};
  }

  a {
    font-weight: 500;
    color: ${theme.colors.fg.link};
    text-decoration: none;

    &:hover,
    &:focus {
      color: ${theme.colors.fg.link};
      text-decoration: underline;
    }

    & code {
      color: ${theme.colors.fg.link};
    }
  }

  img {
    align-self: center;

    /* max-height: 80vh; */
    max-width: 80%;
    padding: 1rem;
  }

  p img {
    display: inline-block;
    padding: 0;
  }

  figure {
    display: flex;
    flex-flow: column nowrap;
    gap: 0.5rem;
    align-items: center;
    margin: 1rem 1.5rem;

    &:has(${SourceCode.selector}) {
      margin-inline: 0;
    }

    @media screen and (width <= 768px) {
      margin: 1rem 0;
    }

    figcaption {
      display: flex;
      flex-flow: column nowrap;
      gap: 0.5rem;
    }
  }

  blockquote {
    padding: 1rem 1rem 1rem 1.5rem;
    margin: 0;
    color: ${theme.colors.fg.muted};
    background-color: ${theme.colors.bg.container};
    border-inline-start: 4px solid ${theme.colors.fg.container};
    border-radius: 3px;
  }

  ul,
  ol {
    display: flex;
    flex-flow: column nowrap;
    gap: 0.2rem;
    padding-inline-start: 1.6rem;

    &:has(li > :is(div, p, pre, ol, ul, figure, img)) {
      gap: 0.5rem;
    }

    li > :is(div, p, pre, ol, ul, figure, img) {
      margin-block-start: 0.5rem;
    }
  }

  /* toctree */
  li > a + ul {
    margin-top: 0.2rem;
  }

  dl {
    display: flex;
    flex-flow: column nowrap;
    gap: 0.5rem;
    margin: 0;

    dt {
      margin: 0;
      font-weight: 600;
    }

    dd {
      margin-inline-start: 1.5rem;
    }
  }

  hr {
    align-self: stretch;
    margin: 0.5rem 1.5rem;
    border: 0.8px solid #abb1bf;
  }

  p {
    &.caption,
    &.sidebar-title,
    &.topic-title {
      margin-top: 1rem;
      font-size: 1.5rem;
      font-weight: 600;
    }
  }

  ${Container.selector} {
    h1 {
      font-size: 1.5rem;
    }

    h2 {
      font-size: 1.2rem;
    }

    h3,
    h4,
    h5,
    h6 {
      font-size: 1rem;
    }

    h1,
    h2,
    h3,
    h4,
    h5,
    h6 {
      margin: 0;
      font-weight: 600;
    }

    p {
      margin: 0;
    }
  }

  article > article {
    padding: 6px 12px;
    margin: 12px;
    border: 1px solid #eee;
    border-radius: 5px;
  }

  article:first-child {
    margin-left: 0;
  }

  article:last-child {
    margin-right: 0;
  }

  ${breakpoint("mobileWidth")} {
    -webkit-text-size-adjust: 100%;
    text-size-adjust: 100%;
  }
`;
