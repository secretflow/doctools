import type { ComponentProps } from "react";
import { styled } from "styled-components";

import { breakpoint, theme } from "../../../theme";

export function Prompt({ type, prompt }: { type?: string; prompt?: string }) {
  if (!type && !prompt) {
    return <div />;
  }
  return (
    <PromptText $type={type}>
      {type === "input" ? "In" : "Out"} {prompt}
    </PromptText>
  );
}

export type PromptProps = ComponentProps<typeof Prompt>;

const PromptText = styled.code<{ $type?: string | undefined }>`
  flex: 0 0 auto;
  min-width: 64px;
  font-size: 0.9rem;
  color: ${theme.colors.fg.muted} !important;
  text-align: right;

  ${breakpoint("mobileWidth")} {
    min-width: unset;
    text-align: left;
  }
`;
