import { faAngleDown, faAngleRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import type { SchemaObject } from "oas/types";
import type { PropsWithChildren, ReactNode } from "react";
import { createContext, useContext, useState } from "react";
import { styled } from "styled-components";

import { theme } from "../../../../../theme";

import { SchemaHeader } from "./SchemaHeader";
import type { QualifiedSchema } from "./types";
import { isSchema } from "./types";

export const SchemaContext = createContext<{
  parents: SchemaObject[];
}>({ parents: [] });

function Parent({
  schema,
  children,
}: PropsWithChildren<Pick<QualifiedSchema, "schema">>) {
  const { parents } = useContext(SchemaContext);
  return (
    <SchemaContext.Provider value={{ parents: [...parents, schema] }}>
      {children}
    </SchemaContext.Provider>
  );
}

function Folder({ header, children }: PropsWithChildren<{ header: ReactNode }>) {
  const { parents } = useContext(SchemaContext);
  const defaultCollapsed = parents.length > 1;
  const [collapsed, setCollapsed] = useState(defaultCollapsed);
  return (
    <Folder.Container>
      {defaultCollapsed ? (
        <Folder.Header
          role="button"
          aria-disabled="false"
          aria-expanded={collapsed ? "false" : "true"}
          onClick={() => setCollapsed((prev) => !prev)}
        >
          <Folder.Icon icon={collapsed ? faAngleRight : faAngleDown} />
          <Folder.Title>{header}</Folder.Title>
        </Folder.Header>
      ) : null}
      {collapsed ? null : <Folder.Content>{children}</Folder.Content>}
    </Folder.Container>
  );
}

Folder.Container = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: ${theme.spacing.xs};
`;

Folder.Header = styled.div`
  display: flex;
  flex-flow: row nowrap;
  gap: ${theme.spacing.xs};
  align-items: center;
  padding: ${theme.spacing.xs};
  cursor: pointer;
  user-select: none;
  background-color: rgb(0 0 0 / 2%);
  border-radius: ${theme.spacing.s};

  &:hover {
    background-color: rgb(0 0 0 / 4%);
  }
`;

Folder.Icon = styled(FontAwesomeIcon)`
  display: block;
  width: 0.9em;
  height: 0.9em;
  color: ${theme.colors.fg.muted};
`;

Folder.Title = styled.p`
  position: relative;
  top: 0.5px;
  margin: 0;
  font-family: ${theme.fonts.sansSerif};
  font-size: 0.9em;
  font-weight: 500;
  line-height: 1;
  color: ${theme.colors.fg.muted};
`;

Folder.Content = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: ${theme.spacing.xs};

  > ${Folder.Container} {
    padding-inline-start: 10px;
    border-inline-start: 1px solid ${theme.colors.fg.container};
  }
`;

function propertyList(schema: QualifiedSchema["schema"]) {
  const fields = Object.entries(schema.properties ?? {}).map(
    ([k, v]) =>
      ({
        name: k,
        schema: v,
        parent: schema,
      }) satisfies QualifiedSchema,
  );
  if (!fields.length) {
    return null;
  }
  return (
    <PropertyList>
      {fields.map((field) => (
        <PropertyListItem key={field.name}>
          <SchemaTree {...field} />
        </PropertyListItem>
      ))}
    </PropertyList>
  );
}

const PropertyList = styled.ul`
  display: flex;
  flex-flow: column nowrap;
  padding: 0;
  margin: 0;
  list-style: none;
`;

const PropertyListItem = styled.li`
  padding: ${theme.spacing.s};
  margin: 0;
  border: 1px solid ${theme.colors.fg.container};
  border-bottom: none;

  &:first-of-type {
    border-top-left-radius: ${theme.spacing.s};
    border-top-right-radius: ${theme.spacing.s};
  }

  &:last-of-type {
    border-bottom: 1px solid ${theme.colors.fg.container};
    border-bottom-right-radius: ${theme.spacing.s};
    border-bottom-left-radius: ${theme.spacing.s};
  }
`;

function arrayItems(schema: QualifiedSchema["schema"]) {
  if (schema.type !== "array" || !isSchema(schema.items)) {
    return null;
  }
  const children = innerSchema(schema.items);
  if (!children) {
    return null;
  }
  return (
    <Parent schema={schema}>
      <Folder
        header={
          <SchemaTree.TitleLabel>
            <Trans>array</Trans>
          </SchemaTree.TitleLabel>
        }
      >
        {children}
      </Folder>
    </Parent>
  );
}

function objectProperties(schema: QualifiedSchema["schema"]) {
  if (schema.type !== "object") {
    return null;
  }
  const knownProperties = propertyList(schema);
  const extraProperties = isSchema(schema.additionalProperties)
    ? innerSchema(schema.additionalProperties)
    : null;
  if (!knownProperties && !extraProperties) {
    return null;
  }
  const title = schema["x-readme-ref-name"];
  return (
    <Parent schema={schema}>
      {knownProperties ? (
        <Folder
          header={
            <Trans>
              <SchemaTree.TitleLabel>object</SchemaTree.TitleLabel>
              {title ? (
                <span>
                  {" "}
                  <SchemaTree.ObjectName>{title}</SchemaTree.ObjectName>
                </span>
              ) : null}
            </Trans>
          }
        >
          {knownProperties}
        </Folder>
      ) : null}
      {extraProperties ? (
        <Folder
          header={
            <SchemaTree.TitleLabel>
              <Trans>map</Trans>
            </SchemaTree.TitleLabel>
          }
        >
          {extraProperties}
        </Folder>
      ) : null}
    </Parent>
  );
}

function innerSchema(schema: QualifiedSchema["schema"]): ReactNode {
  return schema.type === "array" ? arrayItems(schema) : objectProperties(schema);
}

export function SchemaTree({ name, schema, parent }: QualifiedSchema) {
  const inner = innerSchema(schema);
  if (inner === null && name === undefined) {
    return (
      <PropertyList>
        <PropertyListItem style={{ color: theme.colors.fg.muted }}>
          (empty)
        </PropertyListItem>
      </PropertyList>
    );
  }
  return (
    <SchemaTree.Container>
      {name !== undefined ? (
        <SchemaHeader name={name} schema={schema} parent={parent} />
      ) : null}
      {inner}
    </SchemaTree.Container>
  );
}

SchemaTree.Container = styled.section`
  display: flex;
  flex-flow: column nowrap;
  gap: ${theme.spacing.s};
`;

SchemaTree.TitleLabel = styled.span`
  text-transform: uppercase;
`;

SchemaTree.ObjectName = styled.code`
  font-family: ${theme.fonts.monospace};
  font-style: italic;
`;
