/// <reference types="vitest/globals" />

import type { Processor } from 'unified';
import { VFile } from 'vfile';

type StringifierOptions = {
  processor: Processor;
  source: string;
  output: string;
  cwd?: string;
  path?: string;
};

export async function expectToStringifyInto({
  source,
  output,
  processor,
  cwd,
  path,
}: StringifierOptions) {
  const actual = String(
    await processor.process(
      new VFile({
        value: source.trim(),
        cwd,
        path,
      }),
    ),
  ).trim();
  const expected = output.trim();
  expect(actual).toStrictEqual(expected);
}
