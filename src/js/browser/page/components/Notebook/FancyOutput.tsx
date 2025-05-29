import type { ComponentProps } from "react";
import { Fragment } from "react";
import { styled } from "styled-components";

import type { PromptProps } from "./Prompt";
import { Prompt } from "./Prompt";
import { ansi } from "./ansi";

export function FancyOutput({
  type,
  prompt,
  ...innerProps
}: ComponentProps<"div"> & PromptProps) {
  const promptProps = { type, prompt };
  return (
    <Fragment>
      <Prompt {...promptProps} />
      <Output {...innerProps} />
    </Fragment>
  );
}

const Output = styled.div`
  ${ansi}

  > div {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    justify-content: center;

    table {
      font-size: 0.9rem;
    }
  }
`;
