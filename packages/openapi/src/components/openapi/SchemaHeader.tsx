import { Trans, Plural } from '@lingui/macro';
import { useLingui } from '@lingui/react';
import { MDXProvider } from '@mdx-js/react';
import { Popover } from 'antd';
import type { SchemaObject } from 'oas/types';
import { useContext } from 'react';
import styled from 'styled-components';

import { Comma } from '@/components/i18n/punctuations';
import * as markdown from '@/components/markdown/components';
import { MarkdownEval } from '@/components/markdown/MarkdownEval';
import { lightTheme } from '@/theme';
import { intersperse } from '@/utils/itertools';

import { CodeHighlighter } from './CodeHighlighter';
import { SchemaContext } from './SchemaTree';
import { paragraphs, typeExcerpt } from './text';
import { TextualRepresentation } from './TextualRepresentation';
import type { QualifiedSchema } from './types';

function FieldName({ schema, name }: Pick<QualifiedSchema, 'schema' | 'name'>) {
  const { parents } = useContext(SchemaContext);
  const breadcrumbs = pathToSchema(...parents, schema);
  const text = breadcrumbs.reduce((acc, line, y) => `${acc}\n${'  '.repeat(y)}${line}`);
  if (!name) {
    return null;
  }
  return (
    <Popover
      content={<CodeHighlighter language="json">{text}</CodeHighlighter>}
      mouseEnterDelay={1}
      placement="right"
      overlayInnerStyle={{ padding: lightTheme.vars.openapi.spacing.s }}
    >
      <FieldName.Text>{name}</FieldName.Text>
    </Popover>
  );
}

FieldName.Text = styled.code`
  font-family: ${lightTheme.vars.openapi.typography.monospace};
  font-size: 1em;
  font-weight: 700;
  color: ${lightTheme.vars.openapi.colors.default};
  cursor: help;
`;

function TypeExcerpt({ schema }: Pick<QualifiedSchema, 'schema'>) {
  useLingui();
  const text = typeExcerpt(schema);
  if (!text) {
    return null;
  }
  return <TypeExcerpt.Text>{text}</TypeExcerpt.Text>;
}

TypeExcerpt.Text = styled.span`
  display: inline-block;
  margin-inline-start: 0.9ch;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-size: 0.95em;
  font-weight: 600;
  color: ${lightTheme.vars.openapi.colors.blue};
`;

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-required
 */
function Required({ name, parent }: Pick<QualifiedSchema, 'name' | 'parent'>) {
  if (!(name && Array.isArray(parent?.required) && parent.required?.includes(name))) {
    return null;
  }
  return (
    <Required.Text>
      <Trans>required</Trans>
    </Required.Text>
  );
}

Required.Text = styled.em`
  display: inline-block;
  margin-inline-start: 0.9ch;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-size: 0.9em;
  font-weight: 600;
  color: ${lightTheme.vars.openapi.colors.red};
`;

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-default
 */
function Default({ schema }: Pick<QualifiedSchema, 'schema'>) {
  const defaultValue = schema.default;
  if (defaultValue === undefined) {
    return null;
  }
  return (
    <ValidationItem>
      <ValidationType>
        <Trans>default =</Trans>
      </ValidationType>
      <TextualRepresentation value={defaultValue} />
    </ValidationItem>
  );
}

function Example({ schema }: Pick<QualifiedSchema, 'schema'>) {
  const exampleValue = schema.example;
  if (exampleValue === undefined) {
    return null;
  }
  return (
    <ValidationItem>
      <ValidationType>
        <Trans>example:</Trans>
      </ValidationType>
      <TextualRepresentation value={exampleValue} />
    </ValidationItem>
  );
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-enum
 */
function Enum({ schema }: Pick<QualifiedSchema, 'schema'>) {
  if (!schema.enum?.length) {
    return null;
  }
  return (
    <ValidationItem>
      <ValidationType>
        <Trans>one of</Trans>
      </ValidationType>
      {schema.enum.map((v) => (
        <Enum.Item key={v}>
          <TextualRepresentation value={v} />
        </Enum.Item>
      ))}
    </ValidationItem>
  );
}

Enum.Item = styled.span`
  display: inline-block;
  padding: 0.3ch 0.5ch;
  margin-block: 0.3ch;
  margin-inline-end: 0.3ch;
  line-height: 1;
  background-color: ${lightTheme.vars.openapi.backgroundColors.default};
  border: 1px solid ${lightTheme.vars.openapi.colors.border};
  border-radius: 0.5ch;
`;

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-num
 */
function NumericConstraints({ schema }: Pick<QualifiedSchema, 'schema'>) {
  const constraints: React.ReactElement[] = [];
  if (schema.minimum !== undefined) {
    constraints.push(<code key="minimum">{`>=${schema.minimum}`}</code>);
  }
  if (schema.maximum !== undefined) {
    constraints.push(<code key="maximum">{`<=${schema.maximum}`}</code>);
  }
  if (schema.exclusiveMinimum !== undefined) {
    constraints.push(<code key="exclusive-minimum">{'>'}</code>);
  }
  if (schema.exclusiveMaximum !== undefined) {
    constraints.push(<code key="exclusive-maximum">{'<'}</code>);
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
  return <ValidationItem>{intersperse(constraints, inlineSeparator)}</ValidationItem>;
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-str
 */
function StringConstraints({ schema }: Pick<QualifiedSchema, 'schema'>) {
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
  return <ValidationItem>{intersperse(constraints, inlineSeparator)}</ValidationItem>;
}

/**
 * @see https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-arr
 */
function ArrayConstraints({ schema }: Pick<QualifiedSchema, 'schema'>) {
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
function ObjectConstraints({ schema }: Pick<QualifiedSchema, 'schema'>) {
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
  return <ValidationItem>{intersperse(constraints, inlineSeparator)}</ValidationItem>;
}

export function SchemaHeader({ schema, parent, name }: QualifiedSchema) {
  return (
    <SchemaHeader.Container>
      <SchemaHeader.Excerpt>
        <FieldName name={name} schema={schema} />
        <TypeExcerpt schema={schema} />
        <Required name={name} parent={parent} />
      </SchemaHeader.Excerpt>
      <SchemaHeader.Description>
        <MDXProvider components={markdown.prose}>
          {/* FIXME: decide on whether to treat titles as part of the description */}
          <MarkdownEval content={paragraphs('\n')(schema.title, schema.description)} />
        </MDXProvider>
      </SchemaHeader.Description>
      <SchemaHeader.Examples>
        <Default schema={schema} />
        <Example schema={schema} />
      </SchemaHeader.Examples>
      <SchemaHeader.Constraints>
        <Enum schema={schema} />
        <NumericConstraints schema={schema} />
        <StringConstraints schema={schema} />
        <ArrayConstraints schema={schema} />
        <ObjectConstraints schema={schema} />
      </SchemaHeader.Constraints>
    </SchemaHeader.Container>
  );
}

SchemaHeader.Container = styled.div`
  :empty {
    display: none;
  }

  display: flex;
  flex-flow: column nowrap;
  gap: ${lightTheme.vars.openapi.spacing.xs};
  font-family: ${lightTheme.vars.openapi.typography.sans};
`;

SchemaHeader.Head = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: ${lightTheme.vars.openapi.spacing.s};
`;

SchemaHeader.Excerpt = styled.div`
  position: relative;
`;

SchemaHeader.Description = styled.div`
  font-size: 0.95em;
`;

SchemaHeader.Card = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: ${lightTheme.vars.openapi.spacing.xs};
  padding: ${lightTheme.vars.openapi.spacing.xs} ${lightTheme.vars.openapi.spacing.s};
  font-family: ${lightTheme.vars.openapi.typography.monospace};
  color: ${lightTheme.vars.openapi.colors.default};
  border-radius: ${lightTheme.vars.openapi.spacing.xs};
`;

SchemaHeader.Examples = styled(SchemaHeader.Card)`
  background-color: ${lightTheme.vars.openapi.backgroundColors.info};
`;

SchemaHeader.Constraints = styled(SchemaHeader.Card)`
  background-color: ${lightTheme.vars.openapi.backgroundColors.warning};
`;

const ValidationItem = styled.p`
  margin: 0;
  font-size: 0.9em;

  strong {
    font-weight: 500;
  }
`;

const ValidationType = styled.strong`
  margin-inline-end: 0.5ch;
  font-weight: 500;
`;

const inlineSeparator = (i: number) => <Comma key={`sep-${i}`} />;

function pathToSchema(...parents: SchemaObject[]): string[] {
  const lines: string[] = [];
  let parentObject: SchemaObject | undefined = undefined;
  let currentSchema = parents.shift();
  while (currentSchema) {
    let prefix = '';
    if (parentObject?.type === 'object') {
      let key = Object.entries(parentObject?.properties ?? {}).find(
        ([, v]) => v === currentSchema,
      )?.[0];
      if (key === undefined && parentObject?.additionalProperties) {
        key = '*';
      }
      if (key !== undefined) {
        prefix = `${JSON.stringify(key)}: `;
      }
    }
    if (currentSchema.type === 'array') {
      lines.push(`${prefix}[`);
      parentObject = undefined;
    } else if (currentSchema.type === 'object') {
      lines.push(`${prefix}{`);
      parentObject = currentSchema;
    } else {
      lines.push(`${prefix}// ${typeExcerpt(currentSchema)}`);
      parentObject = undefined;
    }
    currentSchema = parents.shift();
  }
  const lastLine = lines.pop();
  if (lastLine?.endsWith('{')) {
    lines.push(`${lastLine} // object`);
  } else if (lastLine?.endsWith('[')) {
    lines.push(`${lastLine} // array`);
  } else if (lastLine) {
    lines.push(lastLine);
  }
  return lines;
}
