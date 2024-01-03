import type { Meta, StoryObj } from '@storybook/react';

import { MarkdownEval } from './MarkdownEval';

const meta: Meta<typeof MarkdownEval> = {
  component: MarkdownEval,
};

export default meta;

type Story = StoryObj<typeof MarkdownEval>;

export const HelloWorld: Story = {
  name: 'Hello, world!',
  render: () => (
    <MarkdownEval
      content={`\
# Hello, world!

Below is an example of markdown in JSX.
      `}
    />
  ),
};

export const Interactive: Story = {
  args: {
    content: '# Hello, world!',
  },
};
