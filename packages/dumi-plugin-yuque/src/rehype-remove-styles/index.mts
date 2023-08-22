import type { Element } from 'hast';
import postcss from 'postcss';
import postcssJs from 'postcss-js';
import type { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

interface Options {
  properties: (string | Record<string, string | string[]>)[];
}

export function rehypeRemoveStyle(options: Options): Transformer {
  const { properties } = options;
  return async (tree) => {
    for (const property of properties) {
      const nodes: Element[] = [];

      // get all nodes by visit(): to solve the async problem,
      // because the third argument cannot be async function
      visit(tree, 'element', (node: Element) => {
        nodes.push(node);
      });

      for (const node of nodes) {
        let _property: string | string[] = '';
        if (typeof property === 'string') {
          _property = property;

          await updateNodeStyle(node, _property);
        } else if (property instanceof Object) {
          _property = property[node.tagName];

          if (_property instanceof Array) {
            for (const _eachProperty of _property) {
              await updateNodeStyle(node, _eachProperty);
            }
          } else {
            await updateNodeStyle(node, _property);
          }
        }
      }
    }
  };
}

async function updateNodeStyle(node: Element, property: string) {
  if (node?.properties?.['style'] && property) {
    const cssObject =
      postcssJs.objectify(postcss.parse(node.properties['style'])) || {};
    delete cssObject[property];

    const result = await postcss().process(cssObject, {
      // @ts-expect-error 见 https://github.com/postcss/postcss-js#compile-css-in-js-to-css
      parser: postcssJs,
    });

    const cssString = result.css;

    if (cssString && node?.properties?.['style']) {
      node.properties['style'] = cssString;
    } else {
      delete node.properties?.['style'];
    }
  }

  if (node.properties?.[property] && property) {
    delete node.properties[property];
  }
}
