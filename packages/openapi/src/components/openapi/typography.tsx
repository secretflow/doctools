import { Typography } from "antd";
import styled from "styled-components";

import { lightTheme } from "@/theme";

export const Copyable = styled(Typography.Text)`
  .ant-typography-copy {
    margin: 0;
    color: ${lightTheme.vars.openapi.colors.neutral};
  }
`;
