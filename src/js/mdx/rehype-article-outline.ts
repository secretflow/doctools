import { fromJs } from "esast-util-from-js";
import Slugger from "github-slugger";
import type * as hast from "hast";
import { toString as hastToString } from "hast-util-to-string";
import { toString as mdastToString } from "mdast-util-to-string";
import type { OpenAPIV2 } from "openapi-types";
import type { Transformer } from "unified";
import type { Parent } from "unist";
import { convert } from "unist-util-is";
import { selectAll } from "unist-util-select";
import { visit } from "unist-util-visit";
import YAML from "yaml";

export type OutlineItem = {
  id: string;
  title: string;
  longTitle: string;
  depth: number;
  order: number;
  content: string;
  tags: string[];
  metadata: Record<string, string>;
};

type MdxJsxFlowElementHAST = hast.ElementContentMap["mdxJsxFlowElement"];
type MdxJsxTextElementHAST = hast.ElementContentMap["mdxJsxTextElement"];

type ContentElement = MdxJsxFlowElementHAST | MdxJsxTextElementHAST | hast.Element;

type JSXElementHAST = MdxJsxFlowElementHAST | MdxJsxTextElementHAST;

// Per MDX specs headings should be inline JSX elements
// Here we relax that restriction to allow block-level JSX elements as well
type JSXHeading = (MdxJsxFlowElementHAST | MdxJsxTextElementHAST) & {
  name: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
};

type JSXOutlineElement = MdxJsxFlowElementHAST & {
  name: "Outline";
};

type JSXOpenAPIViewerElement = MdxJsxFlowElementHAST & {
  name: "OpenAPIViewer";
};

type HASTHeading = hast.Element & {
  tagName: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
};

type PointOfInterest =
  | JSXHeading
  | JSXOutlineElement
  | JSXOpenAPIViewerElement
  | HASTHeading;

declare module "vfile" {
  interface DataMap {
    outline?: OutlineItem[];
  }
}

function jsxStringAttribute(elem: JSXElementHAST, name: string): string | undefined {
  const attr = elem.attributes.find(
    (v) => v.type === "mdxJsxAttribute" && v.name === name,
  );
  if (attr && typeof attr.value === "string") {
    return attr.value;
  }
  return undefined;
}

function jsxLiteralAttribute<T>(
  elem: JSXElementHAST,
  name: string,
  parser: (t: string) => T = JSON.parse,
): T | undefined {
  const attr = elem.attributes.find(
    (v) => v.type === "mdxJsxAttribute" && v.name === name,
  );
  if (
    attr &&
    attr.type === "mdxJsxAttribute" &&
    typeof attr.value === "object" &&
    typeof attr.value?.value === "string"
  ) {
    try {
      return parser(attr.value?.value);
    } catch {
      return undefined;
    }
  }
  return undefined;
}

function isHTMLHeading(elem: ContentElement): elem is HASTHeading {
  return elem.type === "element" && /^h[1-6]$/.test(elem.tagName);
}

function isJSXHeading(elem: ContentElement): elem is JSXHeading {
  return (
    (elem.type === "mdxJsxTextElement" || elem.type === "mdxJsxFlowElement") &&
    elem.name !== null &&
    /^h[1-6]$/.test(elem.name)
  );
}

function isCodeSymbol(elem: ContentElement): elem is JSXOutlineElement {
  return elem.type === "mdxJsxFlowElement" && elem.name === "Outline";
}

function isOpenAPIViewer(elem: ContentElement): elem is JSXOpenAPIViewerElement {
  return elem.type === "mdxJsxFlowElement" && elem.name === "OpenAPIViewer";
}

function elementTitle(elem: PointOfInterest) {
  if (isCodeSymbol(elem)) {
    const fullname = jsxLiteralAttribute<string | null>(elem, "fullname");
    if (!fullname) {
      return;
    }
    return fullname.split(".").pop() ?? fullname;
  } else if (isJSXHeading(elem)) {
    return mdastToString(elem);
  } else {
    return hastToString(elem);
  }
}

function elementId(elem: PointOfInterest) {
  if (isHTMLHeading(elem)) {
    if (elem.properties?.["id"]) {
      return String(elem.properties["id"]);
    }
  } else if (isCodeSymbol(elem)) {
    return jsxLiteralAttribute<string | null>(elem, "target");
  } else if (isJSXHeading(elem)) {
    return jsxStringAttribute(elem, "id");
  }
  return undefined;
}

export function rehypeArticleOutline(): Transformer {
  return (tree, file) => {
    const outline = new Map<PointOfInterest, OutlineItem>();

    const swagger: OutlineItem[] = [];

    // collect all points of interest

    const slugger = new Slugger();

    let lastTopLevelTitle: string | undefined = undefined;
    let lastFQN: string | undefined = undefined;
    let lastDepth = 0;
    let headingCount = 0;

    selectAll(
      [
        'root > mdxJsxFlowElement[name="h1"]',
        'root > mdxJsxFlowElement[name="h2"]',
        'root > mdxJsxFlowElement[name="h3"]',
        'root > mdxJsxFlowElement[name="h4"]',
        'root > mdxJsxFlowElement[name="h5"]',
        'root > mdxJsxFlowElement[name="h6"]',
        'root > mdxJsxTextElement[name="h1"]',
        'root > mdxJsxTextElement[name="h2"]',
        'root > mdxJsxTextElement[name="h3"]',
        'root > mdxJsxTextElement[name="h4"]',
        'root > mdxJsxTextElement[name="h5"]',
        'root > mdxJsxTextElement[name="h6"]',
        'root > element[tagName="h1"]',
        'root > element[tagName="h2"]',
        'root > element[tagName="h3"]',
        'root > element[tagName="h4"]',
        'root > element[tagName="h5"]',
        'root > element[tagName="h6"]',
        'mdxJsxFlowElement[name="Outline"]',
        'mdxJsxFlowElement[name="OpenAPIViewer"]',
        ...(file.basename?.endsWith(".html")
          ? [
              'element[tagName="h1"]',
              'element[tagName="h2"]',
              'element[tagName="h3"]',
              'element[tagName="h4"]',
              'element[tagName="h5"]',
              'element[tagName="h6"]',
            ]
          : []),
      ].join(", "),
      tree,
    ).forEach((node) => {
      const heading = node as PointOfInterest;

      let depth: number;

      if (isOpenAPIViewer(heading)) {
        const schema = jsxStringAttribute(heading, "schema");
        if (schema) {
          try {
            const data: OpenAPIV2.Document = YAML.parse(schema);
            if (data["swagger"] === "2.0") {
              const paths = data.paths;
              Object.entries(paths).forEach(([path, pathData]) => {
                Object.entries(pathData).forEach(([method, methodData]) => {
                  if (typeof methodData !== "object") {
                    return;
                  }
                  const isOperation = (
                    _: typeof methodData,
                  ): _ is OpenAPIV2.OperationObject => {
                    return [
                      "get",
                      "put",
                      "post",
                      "delete",
                      "options",
                      "head",
                      "patch",
                      "trace",
                    ].includes(method);
                  };
                  if (!isOperation(methodData)) {
                    return;
                  }
                  const operationId = methodData.operationId;
                  if (!operationId) {
                    return;
                  }
                  const summary = methodData.summary;
                  const description = methodData.description;
                  const contentHint: string[] = [];
                  if (method) {
                    contentHint.push(method);
                  }
                  if (path) {
                    contentHint.push(path);
                  }
                  if (operationId) {
                    contentHint.push(operationId);
                  }
                  if (summary) {
                    contentHint.push(summary);
                  }
                  if (description) {
                    contentHint.push(description);
                  }
                  const id = operationId;
                  const title = `${summary}`.trim();
                  swagger.push({
                    id,
                    title: title,
                    longTitle: `${title} - ${summary}`,
                    depth: 1,
                    order: headingCount++,
                    content: contentHint.join(", "),
                    metadata: {},
                    tags: [],
                  });
                });
              });
            }
          } catch {
            // ignored
          }
        }
        return;
      }

      if (isHTMLHeading(heading)) {
        depth = Number(heading.tagName[1]);
        lastFQN = undefined;
      } else if (isJSXHeading(heading)) {
        depth = Number(/h([1-6])/.exec(heading.name)?.[1]);
        lastFQN = undefined;
      } else if (isCodeSymbol(heading)) {
        const fqn = jsxLiteralAttribute<string | null>(heading, "target");
        if (!lastFQN) {
          depth = lastDepth + 1;
        } else {
          if (!fqn) {
            return;
          }
          const fqnParts = fqn.split(".");
          const lastFQNParts = lastFQN.split(".");
          const diff = fqnParts.length - lastFQNParts.length;
          depth = lastDepth + diff;
        }
        if (fqn) {
          lastFQN = fqn;
        }
      } else {
        return;
      }

      lastDepth = depth;

      const title = elementTitle(heading);
      if (!title) {
        return;
      }

      if (depth === 1) {
        lastTopLevelTitle = title;
      }

      let id = elementId(heading);
      if (!id) {
        id = slugger.slug(title);
      } else {
        slugger.occurrences[id] = 1;
      }

      heading.data ??= {};
      Object.assign(heading.data, { id });

      let longTitle = title;

      if (depth !== 1 && lastTopLevelTitle) {
        longTitle = `${lastTopLevelTitle} - ${longTitle}`;
      }

      if (isCodeSymbol(heading)) {
        const fullname = jsxLiteralAttribute<string>(heading, "target");
        const objectType = jsxStringAttribute(heading, "objectType");
        if (fullname && objectType) {
          longTitle = `${objectType} ${fullname}`;
        } else if (fullname) {
          longTitle = fullname;
        }
      }

      outline.set(heading, {
        id,
        title,
        longTitle,
        depth,
        order: headingCount++,
        // initialize containers
        metadata: {},
        tags: [],
        content: "",
      });
    });

    let strayContent = "";

    let currentOutline: OutlineItem | undefined = undefined;

    visit(
      tree,
      convert(["mdxJsxFlowElement", "mdxJsxTextElement", "element"] as const),
      (n) => {
        const node = n as unknown as ContentElement;

        if (isHTMLHeading(node) || isJSXHeading(node) || isCodeSymbol(node)) {
          currentOutline = outline.get(node);
        }

        if (!currentOutline) {
          strayContent += hastToString(node);
          strayContent += " ";
          return;
        }

        if (strayContent) {
          currentOutline.content += strayContent;
          currentOutline.content += " ";
          strayContent = "";
        }

        if (isCodeSymbol(node)) {
          const fullname = jsxLiteralAttribute<string | null>(node, "fullname");
          const sourceModule = jsxLiteralAttribute<string | null>(node, "module");
          const description = jsxLiteralAttribute<string | null>(node, "description");
          const domain = jsxStringAttribute(node, "domain");
          const objectType = jsxStringAttribute(node, "objectType");
          const contentHint: string[] = [];
          if (fullname) {
            contentHint.push(fullname);
          }
          if (domain && objectType) {
            // https://www.sphinx-doc.org/en/master/usage/restructuredtext/domains.html
            const DOMAINS: Record<string, string> = {
              py: "Python",
              cpp: "C++",
              c: "C",
              js: "JavaScript",
            };
            contentHint.push(`${DOMAINS[domain] ?? domain} ${objectType}`);
          }
          if (sourceModule) {
            contentHint.push(`in ${sourceModule}`);
          }
          if (description) {
            contentHint.push(`: ${description}`);
          }
          if (contentHint.length) {
            currentOutline.content += contentHint.join(", ");
            currentOutline.content += " ";
          }
        } else if (isHTMLHeading(node) || isJSXHeading(node)) {
          return;
        } else {
          currentOutline.content += hastToString(node);
          currentOutline.content += " ";
        }
      },
    );

    const data = [...outline.values(), ...swagger];

    file.data["outline"] = data;

    const ser = JSON.stringify(data, (k, v) => {
      if (k === "content") {
        return undefined;
      }
      return v;
    });

    const exportStmt = `export const outline = ${ser}`;

    return {
      ...tree,
      children: [
        {
          type: "mdxjsEsm",
          value: exportStmt,
          data: { estree: fromJs(exportStmt, { module: true }) },
        },
        ...(tree as Parent).children,
      ],
    };
  };
}
