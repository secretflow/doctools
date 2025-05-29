import { Typography } from "antd";
import { styled } from "styled-components";

import { theme } from "../../../../../theme";

export const Copyable = styled(Typography.Text)`
  .ant-typography-copy {
    margin: 0;
    color: ${theme.colors.neutral};
  }
`;
