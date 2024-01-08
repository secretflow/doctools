import { Trans } from '@lingui/macro';
import { MDXProvider } from '@mdx-js/react';
import type { CollapseProps } from 'antd';
import { Collapse, Tag } from 'antd';
import type { Operation } from 'oas/operation';
import { isSchema } from 'oas/types';
import type {
  ResponseObject,
  ParameterObject,
  SchemaObject,
  HttpMethods,
} from 'oas/types';
import styled from 'styled-components';

import * as markdown from '@/components/markdown/components';
import { MarkdownEval } from '@/components/markdown/MarkdownEval';
import { lightTheme } from '@/theme';

import { CodeHighlighter } from './CodeHighlighter';
import { useOpenAPIComponents } from './injection';
import { SchemaTree } from './SchemaTree';
import { maybeJSON, paragraphs } from './text';
import { Copyable } from './typography';

type ElementOf<T> = T extends (infer U)[] ? U : never;

type CollapseItem = ElementOf<CollapseProps['items']>;

type MediaTypeExample = ElementOf<
  ElementOf<Operation['responseExamples']>['mediaTypes'][string]
>;

const HTTP_METHOD_COLORS: Record<HttpMethods, [string, string]> = {
  get: ['#61affe', lightTheme.vars.openapi.colors.inverted],
  post: ['#49cc90', lightTheme.vars.openapi.colors.inverted],
  put: ['#fca130', lightTheme.vars.openapi.colors.inverted],
  patch: ['#4bc5ab', lightTheme.vars.openapi.colors.inverted],
  delete: ['#f93e3e', lightTheme.vars.openapi.colors.inverted],
  head: ['#9012fe', lightTheme.vars.openapi.colors.inverted],
  options: ['#0d5aa7', lightTheme.vars.openapi.colors.inverted],
  trace: ['#1a1a1a', lightTheme.vars.openapi.colors.inverted],
};

function Method({ operation }: { operation: Operation }) {
  const [background, foreground] = HTTP_METHOD_COLORS[operation.method];
  return (
    <Method.Text style={{ color: foreground, backgroundColor: background }}>
      {operation.method}
    </Method.Text>
  );
}

Method.Text = styled.span`
  display: inline-block;
  padding: 0.5ch 0.8ch;
  font-size: 0.8em;
  font-weight: 700;
  line-height: 1;
  text-transform: uppercase;
  border-radius: ${lightTheme.vars.openapi.spacing.xs};
`;

function EndpointTitle({ operation }: { operation: Operation }) {
  const { OperationTitle = EndpointTitle.Text } = useOpenAPIComponents();
  const text =
    operation.getSummary().trim() ||
    `${operation.method.toUpperCase()} ${operation.path}}`;
  return (
    <OperationTitle id={operation.getOperationId()}>
      <MDXProvider components={markdown.inline}>
        <MarkdownEval content={text} />
      </MDXProvider>
    </OperationTitle>
  );
}

EndpointTitle.Text = styled.h1`
  margin: 0;
  font-size: 1.6rem;
`;

EndpointTitle.Anchor = styled.a`
  margin-inline-start: 0.5em;
  font-size: 0.8em;
  color: ${lightTheme.vars.openapi.colors.blue};
  text-decoration: none;
  user-select: none;

  &:hover {
    color: ${lightTheme.vars.openapi.colors.link};
    text-decoration: underline;
  }
`;

function Endpoint({ operation }: { operation: Operation }) {
  return (
    <Endpoint.Text>
      <Method operation={operation} />
      <Endpoint.Path>{operation.path}</Endpoint.Path>
      <Copyable copyable={{ text: operation.path, tooltips: false }} />
    </Endpoint.Text>
  );
}

Endpoint.Text = styled.div`
  display: flex;
  flex-flow: row wrap;
  gap: 0.5ch;
  align-items: baseline;
  margin: 1em 0;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-size: 1.5em;
`;

Endpoint.Path = styled.code`
  font-family: ${lightTheme.vars.openapi.typography.monospace};
  font-size: 0.9em;
  font-weight: 400;
  line-height: 1;
  color: ${lightTheme.vars.openapi.colors.default};
  text-decoration: 1px dotted underline;
  text-underline-offset: 0.35em;
  letter-spacing: -0.02em;
`;

function EndpointDocumentation({ operation }: { operation: Operation }) {
  return (
    <EndpointDocumentation.Text>
      <Documentation text={paragraphs()(operation.getDescription())} />
    </EndpointDocumentation.Text>
  );
}

EndpointDocumentation.Text = styled.div`
  font-size: 1rem;
`;

function Documentation({ text }: { text: string }) {
  if (!text) {
    return null;
  }
  return (
    <Documentation.Container>
      <MDXProvider components={markdown.prose}>
        <MarkdownEval content={text} />
      </MDXProvider>
    </Documentation.Container>
  );
}

Documentation.Container = styled.section`
  margin: 1.5em 0;
`;

function CodePreview({ mimeType, example }: { mimeType: string; example: unknown }) {
  const text = maybeJSON(example);
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
  return <CodeHighlighter language={language}>{text}</CodeHighlighter>;
}

function RequestExamples({ operation }: { operation: Operation }) {
  const items = operation
    .getRequestBodyExamples()
    .flatMap(
      (
        { mediaType, examples }: { mediaType: string; examples: MediaTypeExample[] },
        idx,
      ) =>
        examples.flatMap((example, idx2): CollapseItem[] => {
          const key = `${mediaType}-${idx}-${idx2}`;
          const value = maybeJSON(example.value);
          const title = example.title || mediaType;
          const desc = paragraphs()(example.summary, example.description);
          return [
            {
              key,
              label: <RequestExamples.Title>{title}</RequestExamples.Title>,
              children: (
                <SectionList>
                  <Section title={null}>
                    <CodePreview mimeType={mediaType} example={value} />
                  </Section>
                  {desc ? (
                    <Section title={null}>
                      <Documentation text={desc} />
                    </Section>
                  ) : null}
                </SectionList>
              ),
            },
          ];
        }),
    );
  return (
    <Section title={<Trans>Request examples</Trans>}>
      <Accordion
        items={items}
        // @ts-expect-error antd failed to account for bigint
        defaultActiveKey={firstKey(items)}
      />
    </Section>
  );
}

RequestExamples.Title = styled.h4`
  margin: 0;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-weight: 500;
  color: ${lightTheme.vars.openapi.colors.muted};
  user-select: none;
`;

function QueryParameters({ operation }: { operation: Operation }) {
  const params = operation.getParameters().filter((p) => p.in === 'query');
  if (!params.length) {
    return null;
  }
  return (
    <Section title={<Trans>Query parameters</Trans>}>
      <SchemaTree name={undefined} schema={paramsToObject(params)} />
    </Section>
  );
}

function RequestBody({ operation }: { operation: Operation }) {
  const requestBody = (() => {
    const info = operation.getRequestBody('application/json');
    if (!info) {
      return null;
    }
    if (Array.isArray(info)) {
      return info[1];
    }
    return info;
  })();
  if (!requestBody || !isSchema(requestBody.schema)) {
    return null;
  }
  return (
    <Section title={<Trans>Request body</Trans>}>
      <SchemaTree name={undefined} schema={requestBody.schema} />
    </Section>
  );
}

function Response({
  operation,
  statusCode,
  response,
}: {
  operation: Operation;
  statusCode: string;
  response: ResponseObject;
}) {
  const schema = response.content?.['application/json']?.schema;
  const examples = operation
    .getResponseExamples()
    .filter((ex) => ex.status === statusCode)
    .flatMap((ex) => ex.mediaTypes['application/json'] ?? [])
    .flatMap((example, idx) => {
      const value = maybeJSON(example.value);
      const title = example.title || 'Example';
      const desc = paragraphs()(example.summary, example.description);
      return [
        {
          key: idx,
          label: <RequestExamples.Title>{title}</RequestExamples.Title>,
          children: (
            <SectionList>
              <Section title={null}>
                <CodePreview mimeType="application/json" example={value} />
              </Section>
              {desc ? (
                <Section title={null}>
                  <Documentation text={desc} />
                </Section>
              ) : null}
            </SectionList>
          ),
        },
      ];
    });
  return (
    <SectionList>
      {examples.length ? (
        <Section title={<Trans>Response examples</Trans>}>
          <Accordion
            items={examples}
            // @ts-expect-error antd failed to account for bigint
            defaultActiveKey={firstKey(examples)}
          />
        </Section>
      ) : null}
      {schema ? (
        <Section title={<Trans>Response body</Trans>}>
          <SchemaTree name={undefined} schema={schema} />
        </Section>
      ) : null}
    </SectionList>
  );
}

function StatusCode({ code }: { code: string }) {
  const numbered = parseInt(code, 10);
  const color = (() => {
    if (numbered < 200) {
      return lightTheme.vars.openapi.colors.blue;
    }
    if (numbered < 300) {
      return lightTheme.vars.openapi.colors.green;
    }
    if (numbered < 400) {
      return lightTheme.vars.openapi.colors.yellow;
    }
    if (numbered < 500) {
      return lightTheme.vars.openapi.colors.red;
    }
    if (numbered < 600) {
      return lightTheme.vars.openapi.colors.magenta;
    }
    return lightTheme.vars.openapi.colors.neutral;
  })();
  return <StatusCode.Text color={color}>{code}</StatusCode.Text>;
}

StatusCode.Text = styled(Tag)`
  margin-inline-end: ${lightTheme.vars.openapi.spacing.s};
  font-family: ${lightTheme.vars.openapi.typography.monospace};
  font-weight: 600;
  line-height: 1.5;
`;

function Responses({ operation }: { operation: Operation }) {
  const items = operation.getResponseStatusCodes().flatMap((code): CollapseItem[] => {
    if (Number.isNaN(Number(code))) {
      return [];
    }
    const response = operation.getResponseByStatusCode(code);
    if (typeof response !== 'object') {
      return [];
    }
    const key = `${code}`;
    const title = (
      <Response.Title>
        <StatusCode code={code} />
        <span>{response.description}</span>
      </Response.Title>
    );
    return [
      {
        key,
        label: title,
        children: (
          <Response operation={operation} statusCode={code} response={response} />
        ),
      },
    ];
  });
  return (
    <Section title={<Trans>Responses</Trans>}>
      <Accordion items={items} />
    </Section>
  );
}

Response.Title = styled.h4`
  display: flex;
  flex-flow: row nowrap;
  align-items: center;
  margin: 0;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-weight: 500;
  color: ${lightTheme.vars.openapi.colors.muted};
  user-select: none;
`;

export function OperationViewer({ operation }: { operation: Operation }) {
  return (
    <OperationViewer.Container>
      <EndpointTitle operation={operation} />
      <Endpoint operation={operation} />
      <EndpointDocumentation operation={operation} />
      <SectionList>
        <RequestExamples operation={operation} />
        <QueryParameters operation={operation} />
        <RequestBody operation={operation} />
        <Responses operation={operation} />
      </SectionList>
    </OperationViewer.Container>
  );
}

OperationViewer.Container = styled.section``;

function Section({
  title,
  children,
}: React.PropsWithChildren<{ title: React.ReactNode }>) {
  return (
    <Section.Container>
      {title ? <Section.Title>{title}</Section.Title> : null}
      <div>{children}</div>
    </Section.Container>
  );
}

Section.Container = styled.section`
  display: flex;
  flex-flow: column nowrap;
  gap: ${lightTheme.vars.openapi.spacing.s};
`;

Section.Title = styled.h3`
  margin: 0;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-size: 0.9rem;
  font-weight: 600;
  line-height: 1rem;
  color: ${lightTheme.vars.openapi.colors.muted};
  text-transform: uppercase;
`;

const SectionList = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: ${lightTheme.vars.openapi.spacing.m};
`;

const Accordion = styled(Collapse)`
  .ant-collapse-header {
    align-items: center;
  }
`;

function firstKey(items: { key?: React.Key | undefined }[]): [React.Key] | [] {
  const key = items[0]?.key;
  if (key === undefined) {
    return [];
  }
  return [key];
}

function paramsToObject(params: ParameterObject[]): SchemaObject {
  const schema: SchemaObject = { type: 'object', properties: {} };
  const properties = schema.properties ?? {};
  params.forEach((p) => {
    if (isSchema(p.schema)) {
      properties[p.name] = p.schema;
    }
  });
  return schema;
}
