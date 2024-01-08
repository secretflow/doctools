import { type SchemaObject, isSchema as isSchema_ } from 'oas/types';

export type QualifiedSchema = {
  name: string | undefined;
  schema: SchemaObject;
  parent?: SchemaObject | undefined;
};

export type SchemaTypes = Extract<NonNullable<SchemaObject['type']>, string>;

export const isSchema = (schema: unknown): schema is SchemaObject =>
  typeof schema === 'object' && schema !== null && isSchema_(schema);
