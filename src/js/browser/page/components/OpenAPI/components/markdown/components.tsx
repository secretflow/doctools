import { ConfigProvider, Divider } from "antd";
import type { PropsWithChildren, ReactNode } from "react";
import { css, styled } from "styled-components";

import { theme } from "../../../../../theme";
import { code } from "../../../intrinsic";

const blockSpacing = css`
  margin: 6px 0;

  &:first-child {
    margin-block-start: 0;
  }

  &:last-child {
    margin-block-end: 0;
  }
`;

const typography = css`
  font-family: ${theme.fonts.sansSerif};
  line-height: 1.4;
  color: ${theme.colors.fg.default};
`;

type RequiredElements =
  | "a"
  | "blockquote"
  | "code"
  | "del"
  | "em"
  | "h1"
  | "h2"
  | "h3"
  | "h4"
  | "h5"
  | "h6"
  | "hr"
  | "img"
  | "input"
  | "li"
  | "ol"
  | "p"
  | "pre"
  | "section"
  | "strong"
  | "sup"
  | "sub"
  | "table"
  | "tbody"
  | "td"
  | "th"
  | "thead"
  | "tr"
  | "ul";

type RequiredComponents = Record<
  RequiredElements,
  keyof JSX.IntrinsicElements | ((props: PropsWithChildren) => ReactNode)
>;

export const prose = {
  p: styled.p`
    ${blockSpacing}
    ${typography}
  `,
  ul: styled.ul`
    ${blockSpacing}
    padding-inline-start: 1rem;
  `,
  ol: styled.ol`
    ${blockSpacing}
    padding-inline-start: 1rem;
  `,
  li: styled.li`
    ${typography}
  `,
  h1: styled.h1`
    ${blockSpacing}
    ${typography}
    font-size: 1.5rem;
  `,
  h2: styled.h2`
    ${blockSpacing}
    ${typography}
    font-size: 1.25rem;
  `,
  h3: styled.h3`
    ${blockSpacing}
    ${typography}
    font-size: 1.125rem;
  `,
  h4: styled.h4`
    ${blockSpacing}
    ${typography}
    font-size: 1rem;
  `,
  h5: styled.h5`
    ${blockSpacing}
    ${typography}
  `,
  h6: styled.h6`
    ${blockSpacing}
    ${typography}
  `,
  a: styled.a`
    color: ${theme.colors.fg.link};
  `,
  strong: "strong",
  em: "em",
  del: "del",
  code: code,
  sub: "sub",
  sup: "sup",
  blockquote: styled.blockquote`
    ${blockSpacing}
    ${typography}
    padding-left: calc(1rem - 3px);
    border-left: 3px solid ${theme.colors.fg.container};
  `,
  pre: styled.pre`
    ${blockSpacing}
    ${typography}
    padding: ${theme.spacing.xs};
    overflow-x: auto;
    background-color: ${theme.colors.bg.default};
    border-radius: ${theme.spacing.xs};
  `,
  img: "img",
  hr: () => (
    <ConfigProvider theme={{ token: { marginLG: 10 } }}>
      <Divider />
    </ConfigProvider>
  ),
  input: "input",
  section: "section",
  table: () => (
    <div
      style={{
        fontFamily: theme.fonts.sansSerif,
        color: theme.colors.fg.muted,
      }}
    >
      <span>(tables are currently not supported)</span>
    </div>
  ),
  tbody: "tbody",
  td: "td",
  th: "th",
  thead: "thead",
  tr: "tr",
} satisfies RequiredComponents;

export const inline = {
  ...prose,
  p: styled.span`
    ${typography}
  `,
  ul: "span",
  ol: "span",
  li: styled.span`
    ${typography}
  `,
  h1: styled.span`
    ${typography}
  `,
  h2: styled.span`
    ${typography}
  `,
  h3: styled.span`
    ${typography}
  `,
  h4: styled.span`
    ${typography}
  `,
  h5: styled.span`
    ${typography}
  `,
  h6: styled.span`
    ${typography}
  `,
  blockquote: styled.span`
    ${typography}
  `,
  pre: styled.span`
    ${typography}
  `,
  table: () => null,
  hr: () => <span>---</span>,
  section: "span",
} satisfies RequiredComponents;
