import { fromJs } from "esast-util-from-js";
import type { Root } from "mdast";
import type { Transformer } from "unified";

export function remarkEscapeMdx(): Transformer<Root, Root> {
  return (tree) => {
    tree.children.forEach((block) => {
      switch (block.type) {
        case "paragraph": {
          const first = block.children[0];
          if (!first) {
            return;
          }
          switch (first.type) {
            case "text":
              if (/^(import|export)\b/.test(first.value)) {
                const expr = JSON.stringify(first.value);
                block.children[0] = {
                  type: "mdxTextExpression",
                  value: expr,
                  data: {
                    estree: fromJs(expr),
                  },
                };
              }
          }
        }
      }
    });
  };
}
