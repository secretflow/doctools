import { ConfigProvider, Divider } from 'antd';
import styled, { css } from 'styled-components';

import { lightTheme } from '@/theme';

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
  font-family: ${lightTheme.vars.openapi.typography.sans};
  line-height: 1.4;
  color: ${lightTheme.vars.openapi.colors.default};
`;

type RequiredElements =
  | 'a'
  | 'blockquote'
  | 'code'
  | 'del'
  | 'em'
  | 'h1'
  | 'h2'
  | 'h3'
  | 'h4'
  | 'h5'
  | 'h6'
  | 'hr'
  | 'img'
  | 'input'
  | 'li'
  | 'ol'
  | 'p'
  | 'pre'
  | 'section'
  | 'strong'
  | 'sup'
  | 'sub'
  | 'table'
  | 'tbody'
  | 'td'
  | 'th'
  | 'thead'
  | 'tr'
  | 'ul';

type RequiredComponents = Record<RequiredElements, () => React.ReactNode>;

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
    color: ${lightTheme.vars.openapi.colors.link};
  `,
  strong: styled.strong``,
  em: styled.em``,
  del: styled.del``,
  code: styled.code`
    font-family: ${lightTheme.vars.openapi.typography.monospace};
  `,
  sub: styled.sub``,
  sup: styled.sup``,
  blockquote: styled.blockquote`
    ${blockSpacing}
    ${typography}
    padding-left: calc(1rem - 3px);
    border-left: 3px solid ${lightTheme.vars.openapi.colors.border};
  `,
  pre: styled.pre`
    ${blockSpacing}
    ${typography}
    padding: ${lightTheme.vars.openapi.spacing.xs};
    overflow-x: auto;
    background-color: ${lightTheme.vars.openapi.backgroundColors.default};
    border-radius: ${lightTheme.vars.openapi.spacing.xs};
  `,
  img: styled.img``,
  hr: () => (
    <ConfigProvider theme={{ token: { marginLG: 10 } }}>
      <Divider />
    </ConfigProvider>
  ),
  input: styled.input``,
  section: styled.section``,
  table: () => (
    <div
      style={{
        fontFamily: lightTheme.vars.openapi.typography.sans,
        color: lightTheme.vars.openapi.colors.muted,
      }}
    >
      <span>(tables are currently not supported)</span>
    </div>
  ),
  tbody: styled.tbody``,
  td: styled.td``,
  th: styled.th``,
  thead: styled.thead``,
  tr: styled.tr``,
} satisfies RequiredComponents;

export const inline = {
  ...prose,
  p: styled.span`
    ${typography}
  `,
  ul: styled.span``,
  ol: styled.span``,
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
  section: styled.span``,
} satisfies RequiredComponents;
