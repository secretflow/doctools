import { fromJs } from "esast-util-from-js";
import type * as hast from "hast";
import type { Transformer } from "unified";
import type * as unist from "unist";
import { convert } from "unist-util-is";
import { selectAll } from "unist-util-select";
import { EXIT, SKIP, visit } from "unist-util-visit";

/**
 * Replace all <pre>s with Dumi's <SourceCode />.
 *
 * This is a rehype (HAST) plugin but it emits JSX (MDXAST) elements. It cannot be used
 * with rehype-stringify.
 *
 * Turns this:
 *
 * <pre><code className="language-python">from typing import List
 *
 * def foo(bar: List[int]) -> None:
 *    pass
 * </code></pre>
 *
 * or this:
 *
 * <pre data-language="python"><code>from typing import List
 *
 * def foo(bar: List[int]) -> None:
 *    pass
 * </code></pre>
 *
 * into this:
 *
 * <SourceCode lang="python">{"from typing import List\n\ndef foo(bar: List[int]) -> None:\n   pass\n"}</SourceCode>
 */
export function rehypeSourceCode(): Transformer {
  return (tree) => {
    visit(
      tree,
      convert(
        (node): node is hast.Element =>
          node.type === "element" && (node as hast.Element).tagName === "pre",
      ),
      (node, idx = 0, parent: unist.Parent) => {
        // extract code language

        let lang = "python";

        visit(node, "element", (child) => {
          if (child.type !== "element") {
            return;
          }
          const props: string[] = [];
          ["data-language", "dataLanguage", "className"].forEach((attr) => {
            const item = child.properties?.[attr];
            if (Array.isArray(item)) {
              props.push(...item.map(String));
            } else if (item) {
              props.push(String(item));
            }
          });
          for (const prop of props) {
            const extracted = prop.match(/^(language-)?(?<code>.+)$/);
            if (extracted?.groups?.["code"]) {
              lang = extracted.groups["code"];
              return EXIT;
            }
          }
          return;
        });

        // transform node

        lang = lang.toLowerCase();

        if (
          lang === "default" ||
          lang.startsWith("python") ||
          lang.startsWith("ipython")
        ) {
          lang = "python";
        }
        if (lang === "default") {
          lang = "python";
        }
        if (lang?.startsWith("py") || lang?.startsWith("ipython")) {
          lang = "python";
        }
        if (lang === "c++") {
          lang = "cpp";
        }

        const children = node.children;
        const innerText = JSON.stringify(
          selectAll("text", { type: "root", children })
            .map((v) => "value" in v && v.value)
            .join(""),
        );
        const element = {
          type: "mdxJsxFlowElement",
          name: "SourceCode",
          attributes: [
            {
              type: "mdxJsxAttribute",
              name: "lang",
              value: lang,
            },
          ],
          // Dumi expects the children of <SourceCode /> to be a single text node
          children: [
            {
              type: "mdxFlowExpression",
              value: innerText,
              data: {
                estree: fromJs(innerText),
              },
            },
          ],
        };
        parent.children[idx] = element;
        return SKIP;
      },
    );
  };
}
