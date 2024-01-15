import { plural, t } from "@lingui/macro";
import type { SchemaObject } from "oas/types";
import { isSchema } from "oas/types";

export const maybeJSON = (value: unknown) => {
  if (typeof value === "object") {
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  return String(value);
};

export const truncate = (s: string, n: number) =>
  s.slice(0, n - 1) + (s.length > n ? "..." : "");

export const simpleType = (
  type: SchemaObject["type"] | undefined,
  format: string | undefined,
  count: 1 | 2 = 1,
): string => {
  switch (format) {
    case "date-time":
      return plural(count, { one: "date/time", 1: "date/time", other: "dates/times" });
    case "date":
      return plural(count, { one: "date", 1: "date", other: "dates" });
    case "time":
      return t`time`;
    case "duration":
      return plural(count, { one: "duration", 1: "duration", other: "durations" });
    case "email":
      return plural(count, { one: "email", 1: "email", other: "emails" });
    case "hostname":
      return plural(count, { one: "hostname", 1: "hostname", other: "hostnames" });
    case "ipv4":
      return plural(count, {
        one: "IPv4 address",
        1: "IPv4 address",
        other: "IPv4 addresses",
      });
    case "ipv6":
      return plural(count, {
        one: "IPv6 address",
        1: "IPv6 address",
        other: "IPv6 addresses",
      });
    case "uri":
      return plural(count, { one: "URI", 1: "URI", other: "URIs" });
    case "uuid":
      return plural(count, { one: "UUID", 1: "UUID", other: "UUIDs" });
    case "int32":
      return t`int32`;
    case "int64":
      return t`int64`;
    case "float":
      return plural(count, { one: "float", 1: "float", other: "floats" });
    case "double":
      return plural(count, { one: "double", 1: "double", other: "doubles" });
    case "byte":
      return plural(count, {
        one: "base64 string",
        1: "base64 string",
        other: "base64 strings",
      });
    case "binary":
      return t`binary data`;
    case "password":
      return plural(count, { one: "password", 1: "password", other: "passwords" });
    default:
      break;
  }
  switch (type) {
    case "boolean":
      return plural(count, { one: "boolean", 1: "boolean", other: "booleans" });
    case "integer":
      return plural(count, { one: "integer", 1: "integer", other: "integers" });
    case "number":
      return plural(count, { one: "number", 1: "number", other: "numbers" });
    case "object":
      return plural(count, { one: "object", 1: "object", other: "objects" });
    case "string":
      return plural(count, { one: "string", 1: "string", other: "strings" });
    case "array":
      return plural(count, { one: "array", 1: "array", other: "arrays" });
    default:
      break;
  }
  return type ? String(type) : "unknown";
};

export const typeExcerpt = (
  schema: SchemaObject,
  count: 1 | 2 = 1,
): string | undefined => {
  if (!isSchema(schema)) {
    return undefined;
  }
  if (schema.type === "array" && isSchema(schema.items)) {
    const head = simpleType("array", undefined, count);
    const modifier = typeExcerpt(schema.items, 2);
    return t`${head} of ${modifier}`;
  }
  if (
    schema.type === "object" &&
    !schema.properties &&
    typeof schema.additionalProperties === "object" &&
    isSchema(schema.additionalProperties)
  ) {
    const head = plural(count, { one: "map", 1: "map", other: "maps" });
    const modifier = typeExcerpt(schema.additionalProperties, 2);
    return t`${head} of ${modifier}`;
  }
  const text = simpleType(schema.type, schema.format, count);
  return text;
};

export const paragraphs: (
  sep?: string,
) => (...texts: (string | undefined)[]) => string =
  (sep = "\n\n") =>
  (...texts) =>
    texts
      .filter((x) => x !== undefined)
      .map((x) => x?.trim())
      .join(sep)
      .trim();
