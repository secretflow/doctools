import { Skeleton } from "antd";
import { styled } from "styled-components";

export const Loading = styled(Skeleton).attrs({
  title: false,
  paragraph: { rows: 3 },
  round: true,
  active: true,
})`
  max-width: 480px;
  padding: 2px;
  transition: opacity 0.3s;

  .ant-skeleton-paragraph {
    margin: 0;
  }
`;
