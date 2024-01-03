import styled from 'styled-components';

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

  if (shouldShorten && shortened !== text) {
    return (
      <TextualRepresentation.ShortenedText title={text}>
        {shortened}
      </TextualRepresentation.ShortenedText>
    );
  }

  return (
    <TextualRepresentation.NormalText title={text}>
      {text}
    </TextualRepresentation.NormalText>
  );
}

TextualRepresentation.NormalText = styled.code``;

TextualRepresentation.ShortenedText = styled.code`
  cursor: help;
  text-decoration: 0.08em dotted underline;
  text-underline-offset: 0.3em;
`;
