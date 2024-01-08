import complexNesting from '@readme/oas-examples/3.0/json/complex-nesting.json';
import petStore from '@readme/oas-examples/3.0/json/petstore.json';
import starTrek from '@readme/oas-examples/3.0/json/star-trek.json';
import type { Meta, StoryObj } from '@storybook/react';
import OAS from 'oas';
import type { SchemaObject } from 'oas/types';

import { SchemaTree } from './SchemaTree';

const createSchema = async (raw: unknown) => {
  const oas = new OAS(typeof raw === 'string' ? raw : JSON.stringify(raw));
  await oas.dereference({ preserveRefAsJSONSchemaTitle: true });
  return oas;
};

const petStoreSchema = await createSchema(petStore);
const starTrekSchema = await createSchema(starTrek);
const complexNestingSchema = await createSchema(complexNesting);

const meta: Meta<typeof SchemaTree> = {
  component: SchemaTree,
};

export default meta;

type Story = StoryObj<typeof SchemaTree>;

export const Pet: Story = {
  args: {
    schema: petStoreSchema.api.components?.schemas?.['Pet'] as SchemaObject,
  },
};

export const BookSeriesFull: Story = {
  args: {
    schema: starTrekSchema.api.components?.schemas?.['BookSeriesFull'] as SchemaObject,
  },
};

export const ObjectOfEverything: Story = {
  args: {
    schema: complexNestingSchema.api.components?.schemas?.[
      'ObjectOfEverything'
    ] as SchemaObject,
  },
};

export const AdditionalProperties: Story = {
  args: {
    schema: {
      type: 'object',
      properties: {
        foo: { type: 'string' },
        bar: { type: 'string' },
      },
      additionalProperties: {},
    },
  },
};

export const ArrayOfArrays: Story = {
  args: {
    schema: {
      type: 'object',
      properties: {
        'string[][]': {
          type: 'array',
          items: {
            type: 'array',
            items: {
              type: 'string',
            },
          },
        },
        'object[][]': {
          type: 'array',
          items: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                foo: { type: 'string' },
                bar: { type: 'string' },
              },
            },
          },
        },
        'float[][][]': {
          type: 'array',
          items: {
            type: 'array',
            items: {
              type: 'array',
              items: {
                type: 'number',
                format: 'float',
              },
            },
          },
        },
      },
    },
  },
};

export const MapsOfMaps: Story = {
  args: {
    schema: {
      type: 'object',
      properties: {
        'Map<string, Map<string, string>': {
          type: 'object',
          additionalProperties: {
            type: 'object',
            additionalProperties: {
              type: 'string',
            },
          },
        },
        'Map<string, Map<string, object>': {
          type: 'object',
          additionalProperties: {
            type: 'object',
            additionalProperties: {
              type: 'object',
              properties: {
                foo: { type: 'string' },
                bar: { type: 'string' },
              },
            },
          },
        },
        'Map<string, Map<string, Map<string, float>>>': {
          type: 'object',
          additionalProperties: {
            type: 'object',
            additionalProperties: {
              type: 'object',
              additionalProperties: {
                type: 'number',
                format: 'float',
              },
            },
          },
        },
      },
    },
  },
};

export const ConfusingContainer: Story = {
  args: {
    schema: {
      type: 'object',
      properties: {
        data: {
          type: 'object',
          additionalProperties: {
            type: 'array',
            items: {
              type: 'object',
              additionalProperties: {
                type: 'array',
                items: {
                  type: 'object',
                  additionalProperties: {
                    type: 'array',
                    items: {
                      type: 'object',
                      properties: {
                        foo: { type: 'string' },
                        bar: { type: 'string' },
                      },
                      required: ['foo'],
                      additionalProperties: {
                        type: 'array',
                        items: {
                          type: 'object',
                          properties: {
                            foo: { type: 'string' },
                            bar: { type: 'string' },
                          },
                        },
                      },
                    },
                  },
                },
              },
            },
          },
        },
      },
    },
  },
};
