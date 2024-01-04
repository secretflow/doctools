import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { Editor } from '@monaco-editor/react';
import { Button, Drawer, message } from 'antd';
import { useRef, useState } from 'react';
import YAML from 'yaml';

import { OpenAPIViewer } from '@/components/openapi/OpenAPIViewer';

import { Container } from './Container';

export function App({ schema: original }: { schema: unknown }) {
  const [schema, setSchema] = useState(original);
  const [visible, setVisible] = useState(false);
  const content = typeof schema === 'string' ? schema : YAML.stringify(schema);
  const buffer = useRef(content);
  const [msg] = message.useMessage();
  return (
    <I18nProvider i18n={i18n}>
      <Container>
        <Button onClick={() => setVisible(true)}>Edit schema</Button>
        <OpenAPIViewer schema={schema} />
        <Drawer
          open={visible}
          onClose={() => {
            try {
              setSchema(YAML.parse(buffer.current));
              setVisible(false);
            } catch (e) {
              msg.error(String(e));
            }
          }}
          width="60vw"
        >
          <Editor
            language="yaml"
            value={content}
            onChange={(value) => {
              if (value === undefined) {
                return;
              }
              buffer.current = value;
            }}
          />
        </Drawer>
      </Container>
    </I18nProvider>
  );
}
