import { styled } from "styled-components";

export const Notifications = styled.aside`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.5rem;
  width: 100%;
  min-width: 0;

  .ant-alert {
    padding: 6px 12px;
    line-height: 1.4;
  }

  &:empty {
    display: none;
  }
`;
