import { Anchor } from 'antd';
import { useRouteMeta } from 'dumi';
import { useMemo } from 'react';
import styled from 'styled-components';

const DenseOutlineMenu = styled(Anchor)`
  .ant-anchor-link-title {
    line-height: 1.2;
  }
`;

const PaddedTitle = styled.span<{ depth: number }>`
  white-space: normal;
  overflow-wrap: break-word;
  line-height: 1.2;

  display: inline-block;
  padding-left: ${({ depth }) => depth * 0.7}rem;

  max-width: 100%;
`;

export function Outline({
  scrollingElement,
}: {
  scrollingElement?: React.RefObject<HTMLElement>;
}) {
  const meta = useRouteMeta();

  const anchorItems = useMemo(() => {
    return meta.toc.map((item) => {
      return {
        key: item.id,
        href: '#' + item.id,
        title: <PaddedTitle depth={item.depth - 1}>{item.title}</PaddedTitle>,
      };
    });
  }, [meta.toc]);

  return (
    <DenseOutlineMenu
      affix={false}
      getContainer={() => scrollingElement?.current || window}
      items={anchorItems}
    />
  );
}
