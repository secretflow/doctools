import type { Meta, StoryObj } from "@storybook/react";

import { TextualRepresentation } from "./TextualRepresentation";

const meta: Meta<typeof TextualRepresentation> = {
  component: TextualRepresentation,
};

export default meta;

type Story = StoryObj<typeof TextualRepresentation>;

export const String: Story = {
  args: {
    value: "Hello world",
  },
};

export const LongString: Story = {
  args: {
    value: "A quick brown fox jumps over the lazy dog. ".repeat(10),
  },
};

export const Number: Story = {
  args: {
    value: 42,
  },
};

export const Boolean: Story = {
  args: {
    value: true,
  },
};

export const Null: Story = {
  args: {
    value: null,
  },
};

export const Undefined: Story = {
  args: {
    value: undefined,
  },
};

export const Object: Story = {
  args: {
    value: {
      foo: "bar",
      baz: 42,
    },
  },
};
