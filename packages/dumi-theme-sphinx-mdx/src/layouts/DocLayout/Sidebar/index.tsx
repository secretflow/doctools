import { Tree, Radio } from 'antd';
import { history } from 'dumi';
import { useState } from 'react';
import styled from 'styled-components';

import { useFileSystemTree } from './use-filesystem-tree.js';
import { useTableOfContent } from './use-table-of-content.js';

const SidebarContainer = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.5rem;
  align-items: stretch;
`;

const FlexTree = styled(Tree)`
  .ant-tree-indent-unit {
    width: 12px;
  }

  .ant-tree-switcher {
    width: 12px;
  }

  .ant-tree-node-content-wrapper-normal {
    min-width: 0;
  }

  .ant-tree-title {
    line-height: 1;

    span {
      overflow-wrap: break-word;
      hyphens: auto;
      line-height: 1;
    }
  }
`;

export const Sidebar = () => {
  const [sidebarType, setSidebarType] = useState<'filesystem' | 'toctree'>(
    'filesystem',
  );

  const fsTree = useFileSystemTree();
  const tocTree = useTableOfContent();

  return (
    <SidebarContainer>
      <Radio.Group
        style={{
          position: 'sticky',
          top: 0,
          zIndex: 50,
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          textAlign: 'center',
        }}
        value={sidebarType}
        onChange={(e) => setSidebarType(e.target.value)}
        optionType="button"
        buttonStyle="solid"
        options={[
          { label: 'File System', value: 'filesystem' },
          { label: 'Table of Content', value: 'toctree' },
        ]}
      />
      <FlexTree
        onSelect={(_keys, { node, selected }) => {
          if (!selected) {
            return;
          }
          const { selectable, key } = node;
          if (selectable === false) {
            return;
          }
          history.push(String(key));
        }}
        treeData={sidebarType === 'filesystem' ? fsTree : tocTree}
        blockNode
        autoExpandParent
        defaultExpandAll
      />
    </SidebarContainer>
  );
};
