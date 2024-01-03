import petStore from '@readme/oas-examples/3.0/json/petstore.json';
import type { Meta, StoryObj } from '@storybook/react';
import OAS from 'oas';

import { Operation } from './Operation';

const petStoreSchema = new OAS(JSON.stringify(petStore));

await petStoreSchema.dereference({ preserveRefAsJSONSchemaTitle: true });

const meta: Meta<typeof Operation> = {
  component: Operation,
};

export default meta;

type Story = StoryObj<typeof Operation>;

export const GetPet: Story = {
  args: {
    method: 'get',
    path: '/pet/{petId}',
    operation: petStoreSchema.operation('/pet/{petId}', 'get'),
  },
};

export const PostPet: Story = {
  args: {
    method: 'post',
    path: '/pet',
    operation: petStoreSchema.operation('/pet', 'post'),
  },
};
