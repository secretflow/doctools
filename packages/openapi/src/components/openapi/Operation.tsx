import { Collapse, Tag } from 'antd';
import type OASConstructor from 'oas';
import type { MediaTypeObject, ResponseObject } from 'oas/types';
import { Fragment } from 'react';
import SyntaxHighlighter from 'react-syntax-highlighter';
import { github } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import styled from 'styled-components';

import { MarkdownEval } from '../markdown/MarkdownEval';

import { SchemaTree } from './SchemaTree';
import { maybeJSON } from './text';

type OASOperation = ReturnType<typeof OASConstructor.prototype.operation>;

function Endpoint({ method, path }: { method: string; path: string }) {
  return (
    <Endpoint.Text>
      {method.toUpperCase()} <code>{path}</code>
    </Endpoint.Text>
  );
}

Endpoint.Text = styled.p`
  margin: 0;
  font-size: 1.5rem;
  font-weight: bold;
`;

function Documentation({ operation }: { operation: OASOperation }) {
  const documentation: string = [operation.getSummary(), operation.getDescription()]
    .filter(Boolean)
    .join('\n\n')
    .trim();
  if (!documentation) {
    return null;
  }
  return (
    <Documentation.Container>
      <MarkdownEval content={documentation} />
    </Documentation.Container>
  );
}

Documentation.Container = styled.section`
  p {
    margin: 6px 0;
  }
`;

type _MediaTypeExamples = ReturnType<
  typeof OASConstructor.prototype.getOperation
>['responseExamples'][0]['mediaTypes'];

type MediaTypeExample = _MediaTypeExamples extends Record<string, (infer E)[]>
  ? E
  : never;

function ExamplePreview({
  mimeType,
  example,
}: {
  mimeType: string;
  example: MediaTypeExample;
}) {
  const text = maybeJSON(example.value);
  const language = (() => {
    switch (mimeType) {
      case 'application/json':
        return 'json';
      case 'application/xml':
        return 'xml';
      default:
        return mimeType;
    }
  })();
  return (
    <SyntaxHighlighter
      language={language}
      style={github}
      customStyle={{ margin: 0, fontSize: 12 }}
    >
      {text}
    </SyntaxHighlighter>
  );
}

function ExampleRequestBodies({ operation }: { operation: OASOperation }) {
  const examples = operation.getRequestBodyExamples().flatMap((info) => {
    const values: MediaTypeExample[] = info.examples;
    return values.map((e) => ({ mimeType: info.mediaType, ...e }));
  });
  if (!examples.length) {
    return null;
  }
  return (
    <>
      <Operation.SectionHeading>Example request</Operation.SectionHeading>
      <Collapse
        size="small"
        items={examples.map((ex, i) => {
          const documentation = [ex.title, ex.summary, ex.description]
            .filter(Boolean)
            .join('\n\n')
            .trim();
          return {
            key: `${ex.mimeType}-${documentation || i}`,
            label: (
              <span style={{ color: '#4f5a66' }}>
                {ex.title || ex.summary || ex.mimeType}
              </span>
            ),
            children: (
              <Operation.InnerContainer>
                <Operation.SectionHeading>Request body</Operation.SectionHeading>
                <Documentation.Container>
                  <MarkdownEval content={documentation} />
                </Documentation.Container>
                <ExamplePreview mimeType={ex.mimeType} example={ex} />
              </Operation.InnerContainer>
            ),
          };
        })}
      />
    </>
  );
}

function QueryParameters({ operation }: { operation: OASOperation }) {
  const params = (operation.getParametersAsJSONSchema() ?? []).find(
    (k) => k.type === 'path',
  );
  if (!params) {
    return null;
  }
  return (
    <>
      <Operation.SectionHeading>Query Parameters</Operation.SectionHeading>
      <SchemaTree name={undefined} schema={params.schema} />
    </>
  );
}

function RequestBody({ operation }: { operation: OASOperation }) {
  const requestBody = (operation.getParametersAsJSONSchema() ?? []).find(
    (k) => k.type === 'body',
  );
  if (!requestBody) {
    return null;
  }
  return (
    <>
      <Operation.SectionHeading>Request Body</Operation.SectionHeading>
      <SchemaTree name={undefined} schema={requestBody.schema} />
    </>
  );
}

function StatusTag({ statusCode }: { statusCode: number | string }) {
  const status = Number(statusCode);
  if (status >= 200 && status < 300) {
    return (
      <Tag style={{ fontWeight: 600, letterSpacing: '-0.05px' }} color="green">
        {statusCode}
      </Tag>
    );
  }
  if (status >= 300 && status < 400) {
    return <Tag color="warning">{statusCode}</Tag>;
  }
  if (status >= 400 && status < 600) {
    return (
      <Tag style={{ fontWeight: 600, letterSpacing: '-0.05px' }} color="error">
        {statusCode}
      </Tag>
    );
  }
  return (
    <Tag color="#abb1bf" style={{ fontWeight: 600, letterSpacing: '-0.05px' }}>
      {statusCode}
    </Tag>
  );
}

function Responses({ operation }: { operation: OASOperation }) {
  const statusCodes = operation.getResponseStatusCodes();
  type ResponseInfo = {
    statusCode: number | string;
    schema: ResponseObject;
  };
  const responses: ResponseInfo[] = [];
  statusCodes.forEach((code) => {
    const response = operation.getResponseByStatusCode(code);
    if (typeof response === 'object') {
      responses.push({
        statusCode: code,
        schema: response,
      });
    }
  });
  if (!responses.length) {
    return null;
  }
  const getExamples = (status: number | string, mimeType: string) => {
    const exampleByStatus = operation
      .getResponseExamples()
      .find((ex) => ex.status === status);
    if (!exampleByStatus) {
      return [];
    }
    const examples = exampleByStatus.mediaTypes[mimeType] ?? [];
    return examples;
  };
  return (
    <>
      <Operation.SectionHeading>Responses</Operation.SectionHeading>
      <Collapse
        size="small"
        items={responses.map((r) => {
          const content: Record<string, MediaTypeObject> = r.schema.content ?? {};
          const details: React.ReactNode[] = [];
          Object.entries(content).forEach(([mime, schema]) => {
            if (mime === 'application/json') {
              const examples = getExamples(r.statusCode, mime);
              details.push(
                <Fragment key={mime}>
                  <Operation.SectionHeading>
                    Response body: {mime}
                  </Operation.SectionHeading>
                  {examples.length ? (
                    <Collapse
                      size="small"
                      bordered={false}
                      items={[
                        {
                          label: (
                            <Operation.SectionHeading>
                              Examples
                            </Operation.SectionHeading>
                          ),
                          children: (
                            <>
                              {examples.map((ex, i) => {
                                const documentation = [
                                  ex.title,
                                  ex.summary,
                                  ex.description,
                                ]
                                  .filter(Boolean)
                                  .join('\n\n')
                                  .trim();
                                const key = `${mime}-${documentation || i}`;
                                return (
                                  <Fragment key={key}>
                                    <Documentation.Container>
                                      <MarkdownEval content={documentation} />
                                    </Documentation.Container>
                                    <ExamplePreview mimeType={mime} example={ex} />
                                  </Fragment>
                                );
                              })}
                            </>
                          ),
                        },
                      ]}
                    />
                  ) : null}
                  <SchemaTree
                    key={mime}
                    name={undefined}
                    schema={schema.schema ?? {}}
                  />
                </Fragment>,
              );
            }
          });
          return {
            key: r.statusCode,
            label: (
              <div
                style={{
                  display: 'flex',
                  flexFlow: 'row wrap',
                  alignItems: 'baseline',
                }}
              >
                <StatusTag statusCode={r.statusCode} />
                <span style={{ color: '#4f5a66' }}>{r.schema.description}</span>
              </div>
            ),
            children: details ? (
              <Operation.InnerContainer>{details}</Operation.InnerContainer>
            ) : null,
          };
        })}
      />
    </>
  );
}

export function Operation({
  method,
  path,
  operation,
}: {
  method: string;
  path: string;
  operation: OASOperation;
}) {
  return (
    <Operation.Container>
      <Endpoint method={method} path={path} />
      <Documentation operation={operation} />
      <ExampleRequestBodies operation={operation} />
      <QueryParameters operation={operation} />
      <RequestBody operation={operation} />
      <Responses operation={operation} />
    </Operation.Container>
  );
}

Operation.SectionHeading = styled.p`
  margin: 0;
  color: #4f5a66;
  font-size: 0.8em;
  font-weight: 600;
  text-transform: uppercase;
`;

Operation.Container = styled.section`
  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;

  section:empty {
    display: none;
  }

  > ${Operation.SectionHeading} {
    margin-top: 0.6rem;
  }

  font-family:
    Inter,
    system-ui,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Roboto,
    Oxygen,
    Ubuntu,
    Cantarell,
    'Open Sans',
    'Helvetica Neue',
    sans-serif;

  .ant-list-bordered {
    .ant-list-header {
      padding: 0.2em 0.8em;
      font-size: 0.9em;
      color: #4f5a66;
    }
    .ant-list-item {
      padding: 0.5em 0.8em;
    }
  }
`;

Operation.InnerContainer = styled(Operation.Container)`
  ${Operation.SectionHeading} {
    margin-top: 0;
  }
`;
