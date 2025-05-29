import type { PropsWithChildren } from "react";
import { styled } from "styled-components";

import { theme } from "../../../theme";
import { wordBreak } from "../../whitespace";

const Styled = styled.strong`
  color: ${theme.colors.fg.strong};
`;

export function Name({ children }: PropsWithChildren) {
  return <Styled>{wordBreak(children)}</Styled>;
}

Name.selector = Styled;
