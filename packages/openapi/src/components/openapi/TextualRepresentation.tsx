import { Popover } from 'antd';
import styled from 'styled-components';

import { lightTheme } from '@/theme';

import { CodeHighlighter } from './CodeHighlighter';
import { maybeJSON, truncate } from './text';

export function TextualRepresentation({
  value,
  shorten: shouldShorten = typeof value === 'object',
  len = 40,
}: {
  value: unknown;
  shorten?: boolean;
  len?: number;
}) {
  if (value === undefined) {
    return null;
  }
  const text = maybeJSON(value);
  const shortened = truncate(text, len);
  return (
    <Popover
      content={
        <CodeHighlighter language="json">
          {JSON.stringify(value, null, 2)}
        </CodeHighlighter>
      }
      mouseEnterDelay={1}
      overlayInnerStyle={{ padding: lightTheme.vars.openapi.spacing.s }}
    >
      <TextualRepresentation.Text>
        {shouldShorten && shortened !== text ? shortened : text}
      </TextualRepresentation.Text>
    </Popover>
  );
}

TextualRepresentation.Text = styled.code`
  font-family: ${lightTheme.vars.openapi.typography.monospace};
  cursor: help;
`;
