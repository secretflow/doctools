import petStore from '@readme/oas-examples/3.0/json/petstore.json';
import type { Meta, StoryObj } from '@storybook/react';

import { OpenAPIViewer } from './OpenAPIViewer';

const meta: Meta<typeof OpenAPIViewer> = {
  component: OpenAPIViewer,
};

export default meta;

type Story = StoryObj<typeof OpenAPIViewer>;

export const PetStore: Story = {
  args: { schema: petStore },
};
