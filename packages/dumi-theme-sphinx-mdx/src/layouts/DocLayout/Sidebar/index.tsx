import { Tree, Radio } from 'antd';
import { history } from 'dumi';
import { useState } from 'react';
import styled from 'styled-components';

import { useFileSystemTree } from './use-filesystem-tree.js';

import { useNearestManifest } from '~/exports/manifest.js';

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
  const [sidebarType, setSidebarType] = useState<'filesystem' | 'toctree'>('toctree');

  const fsTree = useFileSystemTree();
  const manifest = useNearestManifest();

  let tree = fsTree;
  if (sidebarType === 'toctree' && manifest) {
    tree = [
      { key: manifest.index, title: '(index)', children: manifest.manifest.sidebar },
    ];
  }

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
          { label: 'Table of Content', value: 'toctree' },
          { label: 'File System', value: 'filesystem' },
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
        treeData={tree}
        blockNode
        autoExpandParent
        defaultExpandAll
      />
    </SidebarContainer>
  );
};
