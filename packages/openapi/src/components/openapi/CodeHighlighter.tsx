import SyntaxHighlighter from 'react-syntax-highlighter';
import { atomOneLight } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import styled from 'styled-components';

import { lightTheme } from '@/theme';

import { Copyable } from './typography';

export function CodeHighlighter({
  language,
  children,
}: Pick<React.ComponentProps<typeof SyntaxHighlighter>, 'language' | 'children'>) {
  return (
    <CodeHighlighter.Container>
      <SyntaxHighlighter
        language={language}
        style={atomOneLight}
        customStyle={{
          margin: 0,
          padding: lightTheme.vars.openapi.spacing.xs,
          paddingInlineEnd: lightTheme.vars.openapi.spacing.ml,
          fontSize: '12px',
          maxHeight: '40vh',
          fontFamily: lightTheme.vars.openapi.typography.monospace,
        }}
        codeTagProps={{
          style: {
            fontFamily: lightTheme.vars.openapi.typography.monospace,
            color: lightTheme.vars.openapi.colors.default,
          },
        }}
      >
        {children}
      </SyntaxHighlighter>
      <CodeHighlighter.CopyButton
        copyable={{ text: String(children), tooltips: false }}
      />
    </CodeHighlighter.Container>
  );
}

CodeHighlighter.Container = styled.div`
  position: relative;
`;

CodeHighlighter.CopyButton = styled(Copyable)`
  position: absolute;
  top: ${lightTheme.vars.openapi.spacing.xs};
  right: ${lightTheme.vars.openapi.spacing.xs};
`;
