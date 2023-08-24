import styled from 'styled-components';

import { Content } from './Content.js';
import { Keyword } from './Keyword.js';
import { Name } from './Name.js';
import { Parameter } from './Parameter.js';
import { ParameterList } from './ParameterList.js';
import { ParameterTarget } from './ParameterTarget.js';
import { Prefix } from './Prefix.js';
import { ReturnType } from './ReturnType.js';
import { Signature } from './Signature.js';
import { TypeAnnotation } from './TypeAnnotation.js';

const OutlineSection = styled.section`
  margin: 1rem 0;

  ${Name} {
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

    ${Name} {
      font-size: 1em;
    }
  }
`;

export const Outline = ({
  children,
}: {
  domain?: string;
  objectType?: string;
  children?: React.ReactNode;
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
