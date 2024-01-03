import type { Meta, StoryObj } from '@storybook/react';

import { SchemaHeader } from './SchemaHeader';

const meta: Meta<typeof SchemaHeader> = {
  component: SchemaHeader,
};

export default meta;

type Story = StoryObj<typeof SchemaHeader>;

export const Int32: Story = {
  args: {
    name: 'size',
    schema: {
      type: 'integer',
      format: 'int32',
      minimum: 0,
      maximum: 100,
      multipleOf: 2,
      default: 42,
    },
  },
};

export const String: Story = {
  args: {
    name: 'name',
    required: ['name'],
    schema: {
      type: 'string',
      description: '# Lorem ipsum',
      minLength: 3,
      maxLength: 20,
      pattern: '^[a-zA-Z0-9]*$',
      default: 'John Doe',
    },
  },
};

export const TextOptions: Story = {
  args: {
    name: 'text',
    schema: {
      type: 'string',
      enum: ['foo', 'bar', 'baz'],
      default: 'bar',
    },
  },
};

export const StringArray: Story = {
  args: {
    name: 'names',
    schema: {
      type: 'array',
      items: {
        type: 'string',
      },
      minItems: 1,
      maxItems: 10,
      uniqueItems: true,
      default: ['John Doe', 'Jane Doe'],
    },
  },
};

export const MapStringString: Story = {
  args: {
    name: 'map',
    schema: {
      type: 'object',
      minProperties: 2,
      maxProperties: 10,
      additionalProperties: {
        type: 'string',
      },
    },
  },
};
