import { Trans, Plural } from '@lingui/macro';
import { useLingui } from '@lingui/react';
import { Tag } from 'antd';
import { isSchema, type SchemaObject } from 'oas/types';
import { Fragment } from 'react';
import styled from 'styled-components';

import { Comma } from '@/components/i18n/punctuations';
import { MarkdownEval } from '@/components/markdown/MarkdownEval';
import { intersperse } from '@/utils/itertools';

import type { SchemaTreeOptions } from './SchemaTree';
import { typeExcerpt } from './text';
import { TextualRepresentation } from './TextualRepresentation';

function FieldName({ name }: { name: string | undefined }) {
  if (!name) {
    return null;
  }
  return <FieldName.Text>{name}</FieldName.Text>;
}

FieldName.Text = styled.code`
  font-weight: 700;
`;

function TypeExcerpt({ schema }: { schema: SchemaObject }) {
  useLingui(); // needed for i18n switching to take effect for messages in typeExcerpt

  const text = typeExcerpt(schema);

  if (!text) {
    return null;
  }

  return <TypeExcerpt.Container>{text}</TypeExcerpt.Container>;
}

TypeExcerpt.Container = styled.span`
  color: #666;
  font-size: 0.9em;
`;

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-required
 */
function Required({ required }: { required: boolean }) {
  if (!required) {
    return null;
  }
  return (
    <Required.Text>
      <Trans>required</Trans>
    </Required.Text>
  );
}

Required.Text = styled.em`
  font-size: 0.9em;
  color: #e26a72;
`;

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-default
 */
function Default({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const defaultValue = schema.default;
  if (defaultValue === undefined) {
    return null;
  }
  return (
    <p>
      <Trans>
        default: <TextualRepresentation value={defaultValue} />
      </Trans>
    </p>
  );
}

function Example({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const exampleValue = schema.example;
  if (exampleValue === undefined) {
    return null;
  }
  return (
    <p>
      <Trans>
        example: <TextualRepresentation value={exampleValue} />
      </Trans>
    </p>
  );
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-enum
 */
function Enum({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  if (!schema.enum?.length) {
    return null;
  }
  return (
    <p>
      <Trans>must be one of</Trans>
      <span style={{ display: 'inline-block', marginInlineEnd: 8 }} />
      {schema.enum.map((v) => (
        <Tag
          key={v}
          style={{ color: 'inherit', lineHeight: 1, padding: '0.3ch 0.6ch' }}
        >
          <TextualRepresentation value={v} />
        </Tag>
      ))}
    </p>
  );
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-num
 */
function NumericConstraints({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const constraints: React.ReactElement[] = [];
  if (schema.minimum !== undefined) {
    constraints.push(<span key="minimum">{`>=${schema.minimum}`}</span>);
  }
  if (schema.maximum !== undefined) {
    constraints.push(<span key="maximum">{`<=${schema.maximum}`}</span>);
  }
  if (schema.exclusiveMinimum !== undefined) {
    constraints.push(<span key="exclusive-minimum">{'>'}</span>);
  }
  if (schema.exclusiveMaximum !== undefined) {
    constraints.push(<span key="exclusive-maximum">{'<'}</span>);
  }
  if (schema.multipleOf !== undefined) {
    constraints.push(
      <span key="multiple-of">
        <Trans>multiple of {schema.multipleOf}</Trans>
      </span>,
    );
  }
  if (!constraints.length) {
    return null;
  }
  return (
    <p>
      <code>{intersperse(constraints, inlineSeparator)}</code>
    </p>
  );
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-str
 */
function StringConstraints({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const constraints: React.ReactElement[] = [];
  if (schema.minLength !== undefined) {
    constraints.push(
      <span key="min-length">
        <Plural
          value={schema.minLength}
          one="minimum # character"
          other="minimum # characters"
        />
      </span>,
    );
  }
  if (schema.maxLength !== undefined) {
    constraints.push(
      <span key="max-length">
        <Plural
          value={schema.maxLength}
          one="maximum # character"
          other="maximum # characters"
        />
      </span>,
    );
  }
  if (schema.pattern !== undefined) {
    constraints.push(
      <span key="pattern">
        <Trans>
          must match regular expression <code>{schema.pattern}</code>
        </Trans>
      </span>,
    );
  }
  if (!constraints.length) {
    return null;
  }
  return <p>{intersperse(constraints, inlineSeparator)}</p>;
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-arr
 */
function ArrayConstraints({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const constraints: React.ReactElement[] = [];
  if (schema.minItems !== undefined) {
    constraints.push(
      <span key="min-items">
        <Plural value={schema.minItems} one="minimum # item" other="minimum # items" />
      </span>,
    );
  }
  if (schema.maxItems !== undefined) {
    constraints.push(
      <span key="max-items">
        <Plural value={schema.maxItems} one="maximum # item" other="maximum # items" />
      </span>,
    );
  }
  if (schema.uniqueItems !== undefined) {
    constraints.push(
      <span key="unique-items">
        <Trans>items must be unique</Trans>
      </span>,
    );
  }
  if (!constraints.length) {
    return null;
  }
  // minContains, maxContains
  return <p>{intersperse(constraints, inlineSeparator)}</p>;
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-obj
 */
function ObjectConstraints({ schema }: { schema: SchemaObject }) {
  if (!isSchema(schema)) {
    return null;
  }
  const constraints: React.ReactElement[] = [];
  if (schema.minProperties !== undefined) {
    constraints.push(
      <span key="min-properties">
        <Plural
          value={schema.minProperties}
          one="minimum # property"
          other="minimum # properties"
        />
      </span>,
    );
  }
  if (schema.maxProperties !== undefined) {
    constraints.push(
      <span key="max-properties">
        <Plural
          value={schema.maxProperties}
          one="maximum # property"
          other="maximum # properties"
        />
      </span>,
    );
  }
  if (schema.additionalProperties === false) {
    constraints.push(
      <span key="no-extra">
        <Trans>no extra keys allowed</Trans>
      </span>,
    );
  }
  if (!constraints.length) {
    return null;
  }
  // dependentRequired
  return <p>{intersperse(constraints, inlineSeparator)}</p>;
}

export function SchemaHeader({ schema, name, required }: SchemaTreeOptions) {
  if (!isSchema(schema) || !name) {
    return null;
  }
  return (
    <SchemaHeader.Container>
      <SchemaHeader.Head>
        <SchemaHeader.Excerpt>
          <FieldName name={name} />
          <TypeExcerpt schema={schema} />
          <Required
            required={Boolean(
              name && Array.isArray(required) && required?.includes(name),
            )}
          />
        </SchemaHeader.Excerpt>
        <SchemaHeader.Validation>
          <Enum schema={schema} />
          <NumericConstraints schema={schema} />
          <StringConstraints schema={schema} />
          <ArrayConstraints schema={schema} />
          <ObjectConstraints schema={schema} />
          <Default schema={schema} />
          <Example schema={schema} />
        </SchemaHeader.Validation>
      </SchemaHeader.Head>
      <SchemaHeader.Description>
        <MarkdownEval content={schema.description || ''} />
      </SchemaHeader.Description>
    </SchemaHeader.Container>
  );
}

SchemaHeader.Container = styled.div`
  *:empty {
    display: none;
  }
  p {
    margin: 0;
  }
  display: flex;
  flex-flow: column nowrap;
  gap: 0.5rem;
`;

SchemaHeader.Head = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.3rem;
`;

SchemaHeader.Excerpt = styled.div`
  display: flex;
  flex-flow: row wrap;
  gap: 1ch;
  align-items: baseline;
`;

SchemaHeader.Description = styled.div`
  &:empty {
    display: none;
  }
`;

/**
 * @see https://spec.openapis.org/oas/v3.0.3#properties
 */
SchemaHeader.Validation = styled.div`
  font-size: 0.9em;
  color: #4f5a66;
`;

const inlineSeparator = (i: number) => (
  <Fragment key={`sep-${i}`}>
    <Comma />
  </Fragment>
);
