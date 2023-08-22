import styled from 'styled-components';

import { highlighted } from '~/internals/common/highlighting.js';
import { permalink } from '~/internals/common/permalink.js';

const SignatureContainer = styled('div')`
  margin-block-end: 0.5rem;
`;

const SignatureLine = styled(permalink(highlighted('pre')))`
  display: block;

  width: 100%;
  padding: 0.5rem 0.5rem;

  overflow-wrap: break-word;
  white-space: pre-wrap;

  font-family: ${({ theme }) => theme.typography.code};
`;

export const Signature = ({
  id,
  children,
}: {
  id?: string;
  children?: React.ReactNode;
}) => {
  return (
    <SignatureContainer>
      <SignatureLine id={id}>{children}</SignatureLine>
    </SignatureContainer>
  );
};

Signature.SignatureLine = SignatureLine;
