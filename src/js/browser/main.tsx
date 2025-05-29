import { I18nProvider } from "@lingui/react";
import { Trans } from "@lingui/react/macro";
import { List, Typography } from "antd";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { useLoaderData, useLocation } from "react-router";
import { styled } from "styled-components";

import { projectKey } from "../docs/types";
import type { Project, Sitemap } from "../docs/types";

import type { RepoLoader, SiteExtras } from "./app";
import { createDocumentationSite, IntraLink } from "./app";
import { intlPolyfill } from "./i18n/polyfill";
import { Loading } from "./layout/Loading";
import { RootPadding } from "./layout/RootPadding";
import { StatusBanner } from "./layout/StatusBanner";
import type { NPMVersion } from "./loaders";
import { fuzzyPath, npmLoader } from "./loaders";

const localVersions: { repo: string; ref: string }[] = [];

const pathFormat = fuzzyPath({
  parse: [
    "/:repo/:ref/:lang/*",
    "/:lang/docs/:repo/:ref/*",
    "/:misc/docs/:repo/:ref/*",
    "/:lang/docs/:repo/*",
    "/docs/:repo/:ref/:lang/*",
    "/docs/:repo/:lang/*",
    "/docs/:repo/:ref/*",
    "/docs/:repo/*",
  ],
  build: ({ repo, ref, lang, suffix = "" }) => {
    let prefix: string;
    if (ref && lang) {
      prefix = `/${lang}/docs/${repo}/${ref}/`;
    } else if (lang) {
      prefix = `/${lang}/docs/${repo}/`;
    } else if (ref) {
      prefix = `/docs/${repo}/${ref}/`;
    } else {
      prefix = `/docs/${repo}/`;
    }
    const base = new URL(prefix, "https://example.org");
    const next = new URL(suffix, base);
    return next.pathname;
  },
  check: {
    // allow local versions with non-PEP440 version numbers to be considered
    ref: ({ repo, ref }) =>
      localVersions.some((local) => local.repo === repo && local.ref === ref),
  },
});

const repoLoader = ((): RepoLoader => {
  const remoteDocs = npmLoader({
    providers: (() => {
      const ref = (version: NPMVersion) => {
        let ref: string;
        switch (version.type) {
          case "latest":
            ref = "latest";
            break;
          case "tagged":
            switch (version.pep440.label) {
              case "head":
                ref = "gh-main";
                break;
              default:
                ref = `gh-${version.pep440.raw}`;
                break;
            }
            break;
          case "canonical":
            ref = version.version;
            break;
        }
        return ref;
      };
      return {
        npmmirror({ repo, version, file }) {
          let url = new URL("https://registry.npmmirror.com/");
          url = new URL(`@secretflow/x-${repo}/${ref(version)}/files/`, url);
          url = new URL(file, url);
          return url;
        },
        jsdelivr({ repo, version, file }) {
          let url = new URL("https://cdn.jsdelivr.net/npm/");
          url = new URL(`@secretflow/x-${repo}@${ref(version)}/`, url);
          url = new URL(file, url);
          return url;
        },
      };
    })(),
  });

  return async (req) => {
    const local = Object.values(await localDocs());
    let result: Awaited<ReturnType<RepoLoader>>;
    try {
      const { fetched, pending } = await remoteDocs(req);
      const map: Record<string, Project> = {};
      fetched.forEach((r) => (map[projectKey(r)] = r));
      local.forEach((r) => (map[projectKey(r)] = r));
      result = { fetched: Object.values(map), pending };
    } catch (e) {
      if (!Object.values(local).some((r) => r.repo === req.path.repo)) {
        throw e;
      } else {
        result = { fetched: local, pending: [] };
      }
    }
    return result;
  };
})();

const siteExtras = {
  routes: [
    {
      path: "/",
      loader: localDocs,
      element: <IndexShim />,
      hydrateFallbackElement: (
        <RootPadding>
          <Loading />
        </RootPadding>
      ),
    },
    {
      path: "/docs",
      loader: localDocs,
      element: <IndexShim />,
      hydrateFallbackElement: (
        <RootPadding>
          <Loading />
        </RootPadding>
      ),
    },
  ],
} satisfies SiteExtras;

const RootFrame = styled.div`
  display: flex;
  flex-flow: column nowrap;
  overflow: visible;
`;

const { DocumentationSite, DocumentationTheme, i18n } = createDocumentationSite({
  pathFormat,
  repoLoader,
  siteExtras,
});

Promise.all([intlPolyfill(), localDocs()])
  .then(([, local]) => {
    localVersions.push(
      ...Object.values(local) //
        .map(({ repo, ref }) => ({ repo, ref })),
    );
  })
  .then(() =>
    createRoot(document.getElementById("root")!).render(
      <StrictMode>
        <DocumentationTheme>
          <RootFrame>
            <I18nProvider i18n={i18n}>
              <StatusBanner />
            </I18nProvider>
            <DocumentationSite />
          </RootFrame>
        </DocumentationTheme>
      </StrictMode>,
    ),
  );

async function localDocs() {
  const entrypoint = "/static/index.js";
  try {
    const module = await import(/* @vite-ignore */ `${entrypoint}`);
    const sitemap: Sitemap = module.default;
    return sitemap;
  } catch {
    return {};
  }
}

function IndexShim() {
  const { pathname } = useLocation();
  switch (pathname) {
    case "":
    case "/":
    case "/docs":
      return <IndexPage />;
    default:
      return (
        <RootPadding>
          <Loading />
        </RootPadding>
      );
  }
}

function IndexPage() {
  const sitemap = useLoaderData<typeof localDocs>();
  return (
    <div
      style={{
        padding: "1rem",
        display: "flex",
        flexFlow: "column nowrap",
        gap: "1rem",
        maxWidth: 1024,
      }}
    >
      <Typography.Title level={1} style={{ margin: 0 }}>
        <Trans>Sitemap</Trans>
      </Typography.Title>
      <List
        bordered
        size="small"
        dataSource={(() => {
          const items = Object.entries(sitemap);
          items.sort((a, b) => a[0].localeCompare(b[0]));
          return items;
        })()}
        renderItem={([prefix, project]) => (
          <List.Item key={prefix}>
            <IntraLink to={pathFormat.build(project)}>
              <Typography.Text
                style={{ fontSize: "1rem", fontWeight: 500, color: "inherit" }}
              >
                {project.repo}, {project.ref}, {project.lang}
              </Typography.Text>
            </IntraLink>
          </List.Item>
        )}
      />
    </div>
  );
}
