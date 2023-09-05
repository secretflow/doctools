import { describe, test, expect } from 'vitest';

import { Manifest } from './schema.mjs';

describe('manifest schema', () => {
  test('valid', () => {
    expect(() => {
      Manifest.parse({
        version: '1',
        sidebar: [
          {
            type: 'category',
            label: 'foo',
            items: [
              {
                type: 'doc',
                id: 'foo/bar',
                label: 'bar',
              },
              {
                type: 'link',
                href: 'https://example.com',
                label: 'example',
              },
              {
                type: 'category',
                label: 'baz',
                items: [
                  {
                    type: 'doc',
                    id: 'foo/baz/qux',
                    label: 'qux',
                  },
                ],
                link: {
                  type: 'doc',
                  id: 'foo/baz',
                  label: 'baz',
                },
              },
            ],
          },
          {
            type: 'link',
            href: 'https://example.com',
            label: 'example',
          },
        ],
      });
    }).not.toThrow();
  });
});
