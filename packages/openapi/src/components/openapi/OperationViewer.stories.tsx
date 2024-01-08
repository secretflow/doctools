import petStore from '@readme/oas-examples/3.0/json/petstore.json';
import type { Meta, StoryObj } from '@storybook/react';
import OAS from 'oas';

import { OperationViewer } from './OperationViewer';

const petStoreSchema = new OAS(JSON.stringify(petStore));

await petStoreSchema.dereference({ preserveRefAsJSONSchemaTitle: true });

const meta: Meta<typeof OperationViewer> = {
  component: OperationViewer,
};

export default meta;

type Story = StoryObj<typeof OperationViewer>;

export const GetPet: Story = {
  args: {
    operation: petStoreSchema.operation('/pet/{petId}', 'get'),
  },
};

export const PostPet: Story = {
  args: {
    operation: petStoreSchema.operation('/pet', 'post'),
  },
};
