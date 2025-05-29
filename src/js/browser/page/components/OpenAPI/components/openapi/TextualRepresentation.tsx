import { Popover } from "antd";
import { styled } from "styled-components";

import { theme } from "../../../../../theme";

import { CodeHighlighter } from "./CodeHighlighter";
import { maybeJSON, truncate } from "./text";

export function TextualRepresentation({
  value,
  shorten: shouldShorten = typeof value === "object",
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
        <CodeHighlighter lang="json">{JSON.stringify(value, null, 2)}</CodeHighlighter>
      }
      mouseEnterDelay={1}
      styles={{ body: { padding: theme.spacing.s } }}
    >
      <TextualRepresentation.Text>
        {shouldShorten && shortened !== text ? shortened : text}
      </TextualRepresentation.Text>
    </Popover>
  );
}

TextualRepresentation.Text = styled.code`
  font-family: ${theme.fonts.monospace};
  cursor: help;
`;
