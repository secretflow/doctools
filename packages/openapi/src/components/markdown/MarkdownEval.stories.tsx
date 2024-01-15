import { MDXProvider } from "@mdx-js/react";
import type { Meta, StoryObj } from "@storybook/react";

import { prose, inline } from "./components";
import markdownCheatsheet from "./markdown-cheat-sheet.md?raw";
import { MarkdownEval } from "./MarkdownEval";

const meta: Meta<typeof MarkdownEval> = {
  component: MarkdownEval,
};

export default meta;

type Story = StoryObj<typeof MarkdownEval>;

export const Prose: Story = {
  render: () => (
    <MDXProvider components={prose}>
      <MarkdownEval content={markdownCheatsheet} />
    </MDXProvider>
  ),
};

export const Inline: Story = {
  render: () => (
    <MDXProvider components={inline}>
      <MarkdownEval content={markdownCheatsheet} />
    </MDXProvider>
  ),
};
