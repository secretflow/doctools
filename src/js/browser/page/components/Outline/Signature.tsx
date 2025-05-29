import type { ReactNode } from "react";
import { styled } from "styled-components";

import { theme } from "../../../theme";
import { highlighted, permalink } from "../../anchoring";

const SignatureContainer = styled("div")`
  margin-block: 1rem 0.5rem;
`;

const SignatureLine = styled(permalink(highlighted("pre")))`
  display: block;
  width: 100%;
  font-family: ${theme.fonts.monospace};
  white-space: normal;
`;

export const Signature = ({ id, children }: { id?: string; children?: ReactNode }) => {
  return (
    <SignatureContainer>
      <SignatureLine id={id}>{children}</SignatureLine>
    </SignatureContainer>
  );
};

Signature.SignatureLine = SignatureLine;
