import { plural, t } from '@lingui/macro';
import type { SchemaObject } from 'oas/types';
import { isSchema } from 'oas/types';

export const maybeJSON = (value: unknown) => {
  if (typeof value === 'object') {
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  return String(value);
};

export const truncate = (s: string, n: number) =>
  s.slice(0, n - 1) + (s.length > n ? '...' : '');

export const simpleType = (
  type: SchemaObject['type'] | undefined,
  format: string | undefined,
  count: 1 | 2 = 1,
): string => {
  switch (format) {
    case 'date-time':
      return plural(count, { one: 'date/time', other: 'dates/times' });
    case 'date':
      return plural(count, { one: 'date', other: 'dates' });
    case 'time':
      return t`time`;
    case 'duration':
      return plural(count, { one: 'duration', other: 'durations' });
    case 'email':
      return plural(count, { one: 'email', other: 'emails' });
    case 'hostname':
      return plural(count, { one: 'hostname', other: 'hostnames' });
    case 'ipv4':
      return plural(count, { one: 'IPv4 address', other: 'IPv4 addresses' });
    case 'ipv6':
      return plural(count, { one: 'IPv6 address', other: 'IPv6 addresses' });
    case 'uri':
      return plural(count, { one: 'URI', other: 'URIs' });
    case 'uuid':
      return plural(count, { one: 'UUID', other: 'UUIDs' });
    case 'int32':
      return t`int32`;
    case 'int64':
      return t`int64`;
    case 'float':
      return t`float`;
    case 'double':
      return t`double`;
    case 'byte':
      return plural(count, { one: 'base64 string', other: 'base64 strings' });
    case 'binary':
      return t`binary`;
    case 'password':
      return plural(count, { one: 'password', other: 'passwords' });
    default:
      break;
  }
  switch (type) {
    case 'boolean':
      return plural(count, { one: 'boolean', other: 'booleans' });
    case 'integer':
      return plural(count, { one: 'integer', other: 'integers' });
    case 'number':
      return plural(count, { one: 'number', other: 'numbers' });
    case 'object':
      return plural(count, { one: 'object', other: 'objects' });
    case 'string':
      return plural(count, { one: 'string', other: 'strings' });
    case 'array':
      return plural(count, { one: 'array', other: 'arrays' });
    default:
      break;
  }
  return type ? String(type) : 'unknown';
};

export const typeExcerpt = (
  schema: SchemaObject,
  count: 1 | 2 = 1,
): string | undefined => {
  if (!isSchema(schema)) {
    return undefined;
  }
  if (schema.type === 'array' && isSchema(schema.items)) {
    const head = simpleType('array', undefined, count);
    const modifier = typeExcerpt(schema.items, 2);
    return `${head} of ${modifier}`;
  }
  if (
    schema.type === 'object' &&
    !schema.properties &&
    typeof schema.additionalProperties === 'object' &&
    isSchema(schema.additionalProperties)
  ) {
    const head = plural(count, { one: 'map', other: 'maps' });
    const modifier = typeExcerpt(schema.additionalProperties, 2);
    return `${head} of ${modifier}`;
  }
  let text = simpleType(schema.type, schema.format, count);
  if (schema.title) {
    if (schema.type === 'object') {
      text = `${schema.title} ${text}`;
    } else {
      text = `${text} (${schema.title})`;
    }
  }
  return text;
};
