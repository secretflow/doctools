import type { I18n } from "@lingui/core";
import { msg, plural } from "@lingui/core/macro";
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
  i18n: I18n,
  type: SchemaObject["type"] | undefined,
  format: string | undefined,
  count: 1 | 2 = 1,
): string => {
  switch (format) {
    case "date-time":
      return i18n._(
        msg`${plural(count, { one: "date/time", 1: "date/time", other: "dates/times" })}`,
      );
    case "date":
      return i18n._(msg`${plural(count, { one: "date", 1: "date", other: "dates" })}`);
    case "time":
      return i18n._(msg`time`);
    case "duration":
      return i18n._(
        msg`${plural(count, { one: "duration", 1: "duration", other: "durations" })}`,
      );
    case "email":
      return i18n._(
        msg`${plural(count, { one: "email", 1: "email", other: "emails" })}`,
      );
    case "hostname":
      return i18n._(
        msg`${plural(count, { one: "hostname", 1: "hostname", other: "hostnames" })}`,
      );
    case "ipv4":
      return i18n._(
        msg`${plural(count, {
          one: "IPv4 address",
          1: "IPv4 address",
          other: "IPv4 addresses",
        })}`,
      );
    case "ipv6":
      return i18n._(
        msg`${plural(count, {
          one: "IPv6 address",
          1: "IPv6 address",
          other: "IPv6 addresses",
        })}`,
      );
    case "uri":
      return i18n._(msg`${plural(count, { one: "URI", 1: "URI", other: "URIs" })}`);
    case "uuid":
      return i18n._(msg`${plural(count, { one: "UUID", 1: "UUID", other: "UUIDs" })}`);
    case "int32":
      return i18n._(msg`int32`);
    case "int64":
      return i18n._(msg`int64`);
    case "float":
      return i18n._(
        msg`${plural(count, { one: "float", 1: "float", other: "floats" })}`,
      );
    case "double":
      return i18n._(
        msg`${plural(count, { one: "double", 1: "double", other: "doubles" })}`,
      );
    case "byte":
      return i18n._(
        msg`${plural(count, {
          one: "base64 string",
          1: "base64 string",
          other: "base64 strings",
        })}`,
      );
    case "binary":
      return i18n._(msg`binary data`);
    case "password":
      return i18n._(
        msg`${plural(count, { one: "password", 1: "password", other: "passwords" })}`,
      );
    default:
      break;
  }
  switch (type) {
    case "boolean":
      return i18n._(
        msg`${plural(count, { one: "boolean", 1: "boolean", other: "booleans" })}`,
      );
    case "integer":
      return i18n._(
        msg`${plural(count, { one: "integer", 1: "integer", other: "integers" })}`,
      );
    case "number":
      return i18n._(
        msg`${plural(count, { one: "number", 1: "number", other: "numbers" })}`,
      );
    case "object":
      return i18n._(
        msg`${plural(count, { one: "object", 1: "object", other: "objects" })}`,
      );
    case "string":
      return i18n._(
        msg`${plural(count, { one: "string", 1: "string", other: "strings" })}`,
      );
    case "array":
      return i18n._(
        msg`${plural(count, { one: "array", 1: "array", other: "arrays" })}`,
      );
    default:
      break;
  }
  return type ? String(type) : "unknown";
};

export const typeExcerpt = (
  i18n: I18n,
  schema: SchemaObject,
  count: 1 | 2 = 1,
): string => {
  if (!isSchema(schema)) {
    return "value";
  }
  if (schema.type === "array" && isSchema(schema.items)) {
    const head = simpleType(i18n, "array", undefined, count);
    const modifier = typeExcerpt(i18n, schema.items, 2);
    return i18n._(msg`${head} of ${modifier}`);
  }
  if (
    schema.type === "object" &&
    !schema.properties &&
    typeof schema.additionalProperties === "object" &&
    isSchema(schema.additionalProperties)
  ) {
    const head = i18n._(msg`${plural(count, { one: "map", 1: "map", other: "maps" })}`);
    const modifier = typeExcerpt(i18n, schema.additionalProperties, 2);
    return i18n._(msg`${head} of ${modifier}`);
  }
  const text = simpleType(i18n, schema.type, schema.format, count);
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
