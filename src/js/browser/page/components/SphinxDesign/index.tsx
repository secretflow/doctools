import { Card as AntDesignCard } from "antd";
import type { JSX, PropsWithChildren } from "react";
import { Link } from "react-router";
import { styled } from "styled-components";

import { theme } from "../../../theme";

type ContainerComponent<T = unknown> = (props: PropsWithChildren<T>) => JSX.Element;

const GridContainer: ContainerComponent = styled.div`
  display: grid;
  grid-template-rows: auto;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
  align-items: stretch;

  & > a {
    cursor: pointer;

    &:hover,
    &:active {
      text-decoration: none !important;
    }
  }
`;

const GridRow = ({ children }: PropsWithChildren) => <>{children}</>;

const GridItem = ({ children }: PropsWithChildren) => <>{children}</>;

const CardBody = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;
`;

const StyledCard = styled(AntDesignCard)`
  height: 100%;

  .ant-card-body {
    height: 100%;
    padding: 1rem 0.9rem;
  }

  &:hover {
    border-color: ${theme.colors.fg.link};
    box-shadow: 0 8px 10px 0 rgb(0 0 0 / 15%);
  }

  ${CardBody} {
    blockquote {
      padding: 0.5rem 1rem;
      font-weight: 400;
      line-height: 1.6;
    }
  }
`;

const Card = ({ href, children }: PropsWithChildren<{ href?: string }>) => {
  if (!href) {
    return <StyledCard size="small">{children}</StyledCard>;
  }
  return (
    <Link to={href}>
      <StyledCard size="small">{children}</StyledCard>
    </Link>
  );
};

const CardTitle = styled.div`
  font-size: 1rem;
  font-weight: 600;
  color: ${theme.colors.fg.strong};
`;

const mapping = new Map([
  ["grid-container", GridContainer],
  ["grid-row", GridRow],
  ["grid-item", GridItem],
  ["card", Card],
  ["card-body", CardBody],
  ["card-title", CardTitle],
]);

export function SphinxDesign({
  type = "",
  children,
  ...props
}: PropsWithChildren<{ type?: string }>) {
  const Component = mapping.get(type);
  if (Component) {
    return <Component {...props}>{children}</Component>;
  }
  return <div>{children}</div>;
}
