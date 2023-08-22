import { describe, test, expect } from 'vitest';

import loader from './index.js';

describe('html-preprocessor', () => {
  test('inline', async () => {
    const input = `
---
title: Hello, world!
---
<!DOCTYPE html>
<html><head></head>
<body><pre data-language="rust">fn main() {
  println!("Hello, world!");
}</pre>
</body></html>
    `.trim();
    const expected = `
---
title: Hello, world!
---
<!DOCTYPE html>&#10;<html><head></head>&#10;<body><pre data-language="rust">fn main() {&#10;  println!("Hello, world!");&#10;}</pre>&#10;</body></html>
    `.trim();
    expect(loader(input)).toBe(expected.trim());
  });
});
