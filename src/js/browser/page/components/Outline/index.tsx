import type { ReactNode } from "react";
import { styled } from "styled-components";

import { Content } from "./Content";
import { Keyword } from "./Keyword";
import { Name } from "./Name";
import { Parameter } from "./Parameter";
import { ParameterList } from "./ParameterList";
import { ParameterTarget } from "./ParameterTarget";
import { Prefix } from "./Prefix";
import { ReturnType } from "./ReturnType";
import { Signature } from "./Signature";
import { TypeAnnotation } from "./TypeAnnotation";

const OutlineSection = styled.section`
  ${Name.selector} {
    font-size: 1.2em;
  }

  ${Signature.SignatureLine} {
    font-size: 1.1em;
  }

  ${Content} {
    section {
      margin: 0;
    }

    ${Signature.SignatureLine} {
      font-size: 1em;
    }

    ${Name.selector} {
      font-size: 1em;
    }
  }
`;

export const Outline = ({
  children,
}: {
  domain?: string;
  objectType?: string;
  children?: ReactNode;
}) => {
  return <OutlineSection>{children}</OutlineSection>;
};

Object.assign(Outline, {
  Content,
  Keyword,
  Name,
  Parameter,
  ParameterList,
  ParameterTarget,
  Prefix,
  ReturnType,
  Signature,
  TypeAnnotation,
});
