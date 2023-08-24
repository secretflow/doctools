import { useOutlet } from 'dumi';
import { useRef } from 'react';
import styled from 'styled-components';

import { Outline } from './Outline/index.js';
import { Sidebar } from './Sidebar/index.js';

import { TwitterCard, DocumentRenderer } from '~/exports/index.js';

const Frame = styled.div`
  box-sizing: border-box;

  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;

  display: flex;
  flex-flow: column nowrap;

  min-height: 0;

  * {
    box-sizing: border-box;
  }
`;

const ContentFrame = styled.div`
  flex: 1 1 auto;
  min-height: 0;
  overflow: scroll;
`;

const ColumnFrame = styled.main`
  height: 100%;
  display: flex;

  flex-flow: row nowrap;
  align-items: stretch;
  gap: 1rem;

  padding: 1.5rem 1.5rem;
`;

const SidebarFrame = styled.div`
  flex: 0 0 300px;
  position: sticky;

  overflow-x: hidden;
  overflow-y: scroll;
`;

const OutlineFrame = styled.div`
  flex: 0 0 240px;
  position: sticky;

  overflow-x: hidden;
  overflow-y: scroll;

  margin-top: 2rem;
`;

const DocLayout = () => {
  const outlet = useOutlet();
  const articleRef = useRef<HTMLElement>(null);
  return (
    <Frame>
      <TwitterCard />
      <ContentFrame>
        <ColumnFrame>
          <SidebarFrame>
            <Sidebar />
          </SidebarFrame>
          <DocumentRenderer ref={articleRef}>{outlet}</DocumentRenderer>
          <OutlineFrame>
            <Outline scrollingElement={articleRef} />
          </OutlineFrame>
        </ColumnFrame>
      </ContentFrame>
    </Frame>
  );
};

export default DocLayout;
