import type { useMDXComponents } from "@mdx-js/react";
import type { FunctionComponent } from "react";
import type { Fragment, jsx, jsxs } from "react/jsx-runtime";
import { z } from "zod";

/**
 * build output exports this as the default export, that is:
 *
 * ```ts
 * declare const instances: Sitemap;
 * export default instances;
 * ```
 */
export type Sitemap = { [prefix: string]: Project };

/**
 * each `Project` identifies documentation in a specific **language**
 * from a specific version of a specific project.
 *
 * the [{@link Project.repo repo}, {@link Project.ref ref}, {@link Project.lang lang}]
 * combination is called a "{@link Project.triple triple}"
 */
export type Project = {
  module: {
    /**
     * a mapping of all pages in the documentation
     *
     * keys are file paths (without extensions) that will match against URL pathnames
     */
    sitemap: {
      [path: string]: {
        exports: () => Promise<{
          render: ContentFunction;
        }>;
      };
    };
    /**
     * metadata about this instance, mainly the {@link ManifestV2.sidebar sidebar}
     */
    manifest: Manifest;
    /**
     * the full text search database
     */
    database: () => Promise<Response>;
  };
  kind: "github";
  owner: string;
  repo: string;
  ref: string;
  lang: string;
  triple: [string, string, string];
};

export type ProjectPage = Project["module"]["sitemap"][string];

export type ContentFunction = (args: {
  jsx: typeof jsx;
  jsxs: typeof jsxs;
  Fragment: typeof Fragment;
  useMDXComponents: typeof useMDXComponents;
}) => Promise<{
  /** the article body as a react component */
  default: FunctionComponent;
  /** outline (headings) of the article, will be shown on the right */
  outline?: OutlineItem[];
  /** metadata for each page, injected by the Sphinx builder */
  frontmatter?: FrontMatter;
}>;

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

export type FrontMatter = Partial<{
  git_download_url: string;
  git_last_modified_commit: string;
  git_last_modified_time: string;
  git_origin_url: string;
  git_owner: string;
  git_repo: string;
  git_revision_commit: string;
  git_revision_time: string;
  page_dependencies: PageDependency[];
}>;

type PageDependency = {
  type: "content" | "gettext" | "attachment";
  path: string;
  time?: RevisionTime | ModifiedTime;
};

type RevisionTime = {
  type: "revision";
  time: string;
  commit: string;
};

type ModifiedTime = {
  type: "revision";
  time: string;
};

export type Manifest = z.infer<typeof Manifest>;

export type ManifestV2 = z.infer<typeof ManifestV2>;

export type ManifestV1 = z.infer<typeof ManifestV1>;

export type Sidebar = z.infer<typeof Sidebar>;

export type SidebarItem = z.infer<typeof sidebarItem> & {
  children?: SidebarItem[] | null | undefined;
};

const sidebarItem = z.object({
  kind: z.enum(["doc", "link", "category"]),
  key: z.string(),
  title: z.string(),
});

const SidebarItem: z.ZodType<SidebarItem> = sidebarItem.extend({
  children: z.lazy(() => z.array(SidebarItem).nullish()),
});

const Sidebar = z.array(SidebarItem);

export const ManifestV1 = z.object({
  version: z.literal("1"),
  sidebar: Sidebar,
});

export const ManifestV2 = z.object({
  version: z.literal("2"),
  sidebar: Sidebar,
  projectName: z.string(),
});

export const Manifest = ManifestV1.or(ManifestV2);

type ProjectKey = Pick<Project, "repo"> & Partial<Pick<Project, "ref" | "lang">>;

export function projectKey({ repo, ref, lang }: ProjectKey) {
  return [repo, ref, lang?.replace(/_/g, "-")].filter((x) => x !== undefined).join("/");
}

export function getProject<T extends ProjectKey>(target: ProjectKey, list: T[]) {
  return list.find(
    (p) => p.repo === target.repo && p.ref === target.ref && p.lang === target.lang,
  );
}

export function getPartialProject<T extends ProjectKey>(target: ProjectKey, list: T[]) {
  return list.find(
    (p) =>
      p.repo === target.repo &&
      (!target.ref || target.ref === p.ref) &&
      (!target.lang || target.lang === p.lang),
  );
}

export function fixLocaleTags(list: Project[]): Project[] {
  const fix = (lang: string) => lang.replaceAll(/_/g, "-");
  return list.map((project) => {
    const [repo, ref, lang] = project.triple;
    return {
      ...project,
      lang: fix(project.lang),
      triple: [repo, ref, fix(lang)],
    };
  });
}

export function uniqueProjects<T extends ProjectKey>(items: T[]): T[] {
  const keys = new Set<string>();
  return items.filter((r) => {
    const k = projectKey(r);
    if (keys.has(k)) {
      return false;
    } else {
      keys.add(k);
      return true;
    }
  });
}
