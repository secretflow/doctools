import { Card as AntDesignCard } from 'antd';
import { Link } from 'dumi';
import styled from 'styled-components';

type ContainerComponent<T = unknown> = (
  props: React.PropsWithChildren<T>,
) => JSX.Element;

const GridContainer: ContainerComponent = styled.div`
  display: grid;
  grid-template-rows: auto;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  align-items: stretch;
  gap: 1rem;

  & > a {
    cursor: pointer;

    &:hover,
    &:active {
      // FIXME:
      text-decoration: none !important;
    }
  }
`;

const GridRow = ({ children }: React.PropsWithChildren) => <>{children}</>;

const GridItem = ({ children }: React.PropsWithChildren) => <>{children}</>;

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
    border-color: ${({ theme }) => theme.colors.link};
    box-shadow: 0 8px 10px 0 rgb(0 0 0 / 15%);
  }

  ${CardBody} {
    blockquote {
      line-height: 1.6;
      font-weight: 400;
      padding: 0.5rem 1rem;
    }
  }
`;

const Card = ({ href, children }: React.PropsWithChildren<{ href?: string }>) => {
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
  color: ${({ theme }) => theme.colors.strong};
  font-size: 1rem;
  font-weight: 600;
`;

const mapping: Map<string, ContainerComponent> = new Map([
  ['grid-container', GridContainer],
  ['grid-row', GridRow],
  ['grid-item', GridItem],
  ['card', Card],
  ['card-body', CardBody],
  ['card-title', CardTitle],
]);

export function SphinxDesign({
  type = '',
  children,
  ...props
}: React.PropsWithChildren<{ type?: string }>) {
  const Component = mapping.get(type);
  if (Component) {
    return <Component {...props}>{children}</Component>;
  }
  return <div>{children}</div>;
}
