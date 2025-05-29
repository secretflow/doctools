import type { PropsWithChildren } from "react";
import { Fragment } from "react";
import { styled } from "styled-components";

import { breakpoint, theme } from "../../../theme";
import { SourceCode } from "../SourceCode";

import type { PromptProps } from "./Prompt";
import { Prompt } from "./Prompt";
import { ansi } from "./ansi";

const CellContent = styled.div`
  display: flex;
  flex: 1 1 auto;
  flex-flow: column;
  min-width: 0;
  max-width: 100%;
  overflow: auto;
  border: 1px solid #ebebeb;

  ${SourceCode.selector} {
    padding: 0;
  }

  pre {
    padding: 12px 18px;
    font-size: 0.9rem;
    line-height: 1.5;

    code {
      color: ${theme.colors.fg.default};
    }

    ${ansi}
  }

  > div {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    justify-content: center;

    ${breakpoint("mobileWidth")} {
      margin-inline-start: 0;
    }

    table {
      font-size: 0.9rem;
    }
  }
`;

export const CodeArea = ({ children, ...prompt }: PropsWithChildren<PromptProps>) => {
  return (
    <Fragment>
      <Prompt {...prompt} />
      <CellContent>{children}</CellContent>
    </Fragment>
  );
};
