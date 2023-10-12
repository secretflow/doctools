import type { DefaultTheme } from 'styled-components';
import { css } from 'styled-components';

export const defaultTokens: DefaultTheme = {
  colors: {
    text: '#323232',
    link: '#0060e6',
    strong: '#e83e8c',
    border: '#e7e7e7',
    container: {
      border: '#b8b8b8',
      text: '#424242',
      background: '#f6f6f6',
    },
    highlight: '#fbe54e',
  },
  typography: {
    text: {
      fontFamily:
        "Inter, Noto Sans SC, Noto Sans, -apple-system, BlinkMacSystemFont, Helvetica Neue, Segoe UI, Roboto, Arial, sans-serif, 'Apple Color Emoji', Segoe UI Emoji, Segoe UI Symbol, Noto Color Emoji",
    },
    code: {
      fontFamily:
        'Fira Code, Inconsolata, PT Mono, SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace',
    },
  },
};

const dumiDefaultSourceCode = css`
  max-width: 100%;

  & > pre {
    font-size: 0.9rem;
    font-family: ${(props) => props.theme.typography.code.fontFamily};
  }

  & > pre.prism-code {
    padding: 12px 18px;
    line-height: 1.5;
  }
`;

export const typesetting = css`
  * {
    overflow-wrap: break-word;
  }

  h1 {
    line-height: 1.5;
    font-size: 2.5rem;
    margin: 1.5rem 0 0rem;

    & + h2 {
      margin-top: 0;
    }
  }

  h2 {
    line-height: 1.5;
    font-size: 1.6rem;
    margin: 1.5rem 0 0rem;
  }

  h3 {
    line-height: 1.4;
    font-size: 1.4rem;
    margin: 1rem 0 0;
  }

  h4 {
    line-height: 1.4;
    font-size: 1.2rem;
    margin: 0.5rem 0 0;
  }

  h5 {
    font-size: 1rem;
    margin: 0;
    font-weight: 500;
  }

  h6 {
    font-size: 1rem;
    margin: 0;
    font-style: italic;
    font-weight: 500;
  }

  em {
    font-weight: 500;
  }

  strong {
    font-weight: 700;
  }

  p,
  pre,
  ul,
  ol {
    margin: 0;
  }

  pre,
  code {
    font-family: ${(props) => props.theme.typography.code.fontFamily};
  }

  code {
    color: ${(props) => props.theme.colors.strong};
  }

  a {
    font-weight: 500;

    text-decoration: none;
    color: ${(props) => props.theme.colors.link};

    &:hover,
    &:focus {
      text-decoration: underline;
      color: ${(props) => props.theme.colors.link};
    }

    & code {
      color: ${(props) => props.theme.colors.link};
    }
  }

  img {
    /* max-height: 80vh; */
    max-width: 80%;
    padding: 1rem;
    align-self: center;
  }

  p img {
    display: inline-block;
    padding: 0;
  }

  figure {
    display: flex;
    flex-flow: column nowrap;
    align-items: center;
    gap: 0.5rem;

    margin: 1rem 1.5rem;

    @media screen and (max-width: 768px) {
      margin: 1rem 0;
    }

    figcaption {
      display: flex;
      flex-flow: column nowrap;
      gap: 0.5rem;
    }

    > .dumi-default-source-code {
      width: 100%;
    }
  }

  blockquote {
    margin: 0;
    padding: 1rem 1rem 1rem 1.5rem;

    border-radius: 3px;
    border-inline-start: 4px solid ${(props) => props.theme.colors.container.border};

    color: ${(props) => props.theme.colors.container.text};
    background-color: ${(props) => props.theme.colors.container.background};
  }

  ul,
  ol {
    display: flex;
    flex-flow: column nowrap;

    padding-inline-start: 1.6rem;
    gap: 0.6rem;

    ul,
    ol {
      gap: 0.4rem;
    }
  }

  // toctree
  li > a + ul {
    margin-top: 0.5rem;
  }

  dl {
    margin: 0;

    display: flex;
    flex-flow: column nowrap;
    gap: 0.5rem;

    dt {
      font-weight: 600;
      margin: 0;
    }

    dd {
      margin-inline-start: 1.5rem;
    }
  }

  hr {
    border: 0.8px solid #abb1bf;
    align-self: stretch;
    margin: 0.5rem 1.5rem;
  }

  table {
    empty-cells: show;
    border: 1px solid ${(props) => props.theme.colors.border};
    border-collapse: collapse;
    border-spacing: 0;

    tbody {
      overflow: auto;
    }

    th,
    td {
      padding: 6px 12px;
      text-align: left;
      border: 1px solid ${(props) => props.theme.colors.border};
    }

    th {
      font-weight: 500;
      white-space: nowrap;
      background-color: ${(props) => props.theme.colors.container.background};
    }

    tbody tr {
      transition: all 0.3s;

      &:hover {
        background: rgba(60, 90, 100, 0.04);
      }
    }

    p,
    span {
      margin: 0;
    }
  }

  p {
    &.caption,
    &.sidebar-title,
    &.topic-title {
      font-weight: 600;
      font-size: 1.5rem;
      margin-top: 1rem;
    }
  }

  .dumi-default-table {
    // No word break for CJK text in tables
    // Tables have automatic scrolling and we want to prevent
    // CJK text from becoming effectively vertical
    word-break: keep-all;
  }

  .dumi-default-source-code {
    ${dumiDefaultSourceCode}
  }

  .dumi-default-table,
  .dumi-default-container {
    margin: 0 !important;

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
      font-weight: 700;
    }

    p {
      margin: 0;
    }

    .dumi-default-source-code {
      ${dumiDefaultSourceCode}
    }
  }

  article > article {
    border-radius: 5px;
    padding: 6px 12px;
    margin: 12px;
    border: 1px solid #eee;
  }

  article:first-child {
    margin-left: 0;
  }

  article:last-child {
    margin-right: 0;
  }
`;
