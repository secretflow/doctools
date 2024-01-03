import { Trans } from '@lingui/macro';
import { List, ConfigProvider } from 'antd';
import { isSchema, type SchemaObject } from 'oas/types';
import styled from 'styled-components';

import { SchemaHeader } from './SchemaHeader';
import { typeExcerpt } from './text';

export type SchemaTreeOptions = {
  schema: SchemaObject;
  name: string | undefined;
  required?: SchemaObject['required'];
};

function Properties({
  header,
  schema,
}: SchemaTreeOptions & { header?: React.ReactNode }) {
  if (!isSchema(schema) || schema.type !== 'object') {
    return null;
  }

  const fields: SchemaTreeOptions[] = Object.entries(schema.properties ?? {}).map(
    ([k, v]) => ({
      name: k,
      schema: v,
      required: schema.required,
    }),
  );

  const footer = (() => {
    if (!schema.additionalProperties) {
      return null;
    }
    if (
      schema.additionalProperties === true ||
      (typeof schema.additionalProperties === 'object' &&
        Object.keys(schema.additionalProperties).length === 0)
    ) {
      if (!fields.length) {
        return null;
      }
      return <Properties.Title>Extra keys allowed</Properties.Title>;
    }
    if (
      isSchema(schema.additionalProperties) &&
      schema.additionalProperties.type === 'object'
    ) {
      return <Properties schema={schema.additionalProperties} name={undefined} />;
    }
    return null;
  })();

  if (!fields.length && !footer) {
    return null;
  }

  return (
    <ConfigProvider renderEmpty={() => <span></span>}>
      <List<SchemaTreeOptions>
        bordered
        size="small"
        header={header}
        footer={footer}
        dataSource={fields}
        renderItem={(item) => (
          <List.Item key={item.name}>
            <SchemaTree {...item} />
          </List.Item>
        )}
      />
    </ConfigProvider>
  );
}

Properties.Title = styled.p`
  margin: 0;
  font-weight: 600;
  text-transform: uppercase;
`;

export function SchemaTree({ name, schema, required }: SchemaTreeOptions) {
  if (!isSchema(schema)) {
    return null;
  }

  const innerContent = (() => {
    if (schema.type === 'object') {
      const header = <Properties.Title>{typeExcerpt(schema)}</Properties.Title>;
      return <Properties schema={schema} name={undefined} header={header} />;
    }

    if (schema.type === 'array' && isSchema(schema.items)) {
      const header = (
        <Properties.Title>
          <Trans>Each item: {typeExcerpt(schema.items)}</Trans>
        </Properties.Title>
      );
      return <Properties schema={schema.items} name={undefined} header={header} />;
    }

    return null;
  })();

  return (
    <SchemaTree.Container>
      <SchemaHeader schema={schema} name={name} required={required} />
      {innerContent}
    </SchemaTree.Container>
  );
}

SchemaTree.Container = styled.section`
  display: flex;
  flex-flow: column nowrap;
  justify-content: flex-start;
  align-items: stretch;
  gap: 0.6rem 0;

  min-width: 0;
  width: 100%;
`;
