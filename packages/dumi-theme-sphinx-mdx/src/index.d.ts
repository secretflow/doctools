import 'dumi';
import 'styled-components';

import type { History } from '@umijs/renderer-react';
import type { StyledInterface } from 'styled-components';

declare module 'dumi' {
  export const history: History;
  export {
    useAppData,
    useOutlet,
    useLocation,
    Helmet,
    Link,
  } from '@umijs/renderer-react';
}

declare module 'styled-components' {
  const styled: StyledInterface;
  export = styled;
}
