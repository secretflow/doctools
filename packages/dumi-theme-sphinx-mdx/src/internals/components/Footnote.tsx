import { Fragment } from 'react';
import styled from 'styled-components';

import { highlighted } from '~/internals/common/highlighting.js';

const FootnoteLabel = styled.em`
  font-weight: 500;
  margin-inline-end: 0.5em;
`;

const FootnoteContainer = styled(highlighted('aside'))`
  // hanging indent
  padding-inline-start: 2rem;
  text-indent: -2rem;

  > p:first-of-type {
    display: inline;
  }
`;

export function Footnote({
  id,
  backrefs,
  label,
  children,
}: React.PropsWithChildren<{
  id: string;
  backrefs?: string[];
  label?: string;
}>) {
  if (!backrefs?.length) {
    return (
      <FootnoteContainer id={id}>
        <FootnoteLabel>[{label || id}]</FootnoteLabel>
        {children}
      </FootnoteContainer>
    );
  }

  if (backrefs?.length === 1) {
    return (
      <FootnoteContainer id={id}>
        <FootnoteLabel>
          [
          <a href={`#${backrefs[0]}`} style={{ fontWeight: 'inherit' }}>
            {label || id}
          </a>
          ]
        </FootnoteLabel>
        {children}
      </FootnoteContainer>
    );
  }

  return (
    <FootnoteContainer id={id}>
      <FootnoteLabel>
        [{label || id}]{' '}
        {backrefs.map((refid, idx) => (
          <Fragment key={refid}>
            <a href={`#${refid}`}>^{idx + 1}</a>{' '}
          </Fragment>
        ))}
      </FootnoteLabel>
      {children}
    </FootnoteContainer>
  );
}
