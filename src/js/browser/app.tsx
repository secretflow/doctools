import { match as matchLocale } from "@formatjs/intl-localematcher";
import type { I18n } from "@lingui/core";
import { Trans } from "@lingui/react/macro";
import { useMDXComponents } from "@mdx-js/react";
import type { Pep440Version } from "@renovatebot/pep440";
import {
  explain as explainPEP440,
  compare as comparePEP440,
} from "@renovatebot/pep440";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Divider } from "antd";
import { debounce } from "lodash";
import type { ComponentProps, PropsWithChildren, ReactNode } from "react";
import { createContext, memo, useContext, useEffect, useRef, useState } from "react";
// import { createRoot } from 'react-dom/client';
// import { renderToString } from 'react-dom/server';
import { HelmetProvider } from "react-helmet-async";
import { Panel, PanelGroup } from "react-resizable-panels";
import type {
  LoaderFunctionArgs,
  PatchRoutesOnNavigationFunctionArgs,
  RouteObject,
  DOMRouterOpts,
} from "react-router";
import {
  createBrowserRouter,
  createPath as routerBuildPath,
  NavLink,
  Outlet,
  parsePath as routerParsePath,
  redirect,
  RouterProvider,
  useLoaderData,
  useLocation,
  useNavigate,
  useNavigation,
} from "react-router";
import { stableHash } from "stable-hash";
import { styled } from "styled-components";

import {
  fixLocaleTags,
  getPartialProject,
  getProject,
  uniqueProjects,
} from "../docs/types";
import type {
  ContentFunction,
  FrontMatter,
  Project,
  ProjectPage,
  Sidebar,
  SidebarItem,
} from "../docs/types";

import { IsNotebook } from "./banners/IsNotebook";
import { IsPrerelease } from "./banners/IsPrerelease";
import { NoSuchVersion } from "./banners/NoSuchVersion";
import { NotTranslated } from "./banners/NotTranslated";
import { Notifications } from "./banners/Notifications";
import { RedirectedFrom } from "./banners/RedirectedFrom";
import { SuggestLanguage } from "./banners/SuggestLanguage";
import type { ErrorContextImpl, ErrorLoggingImpl, ErrorCause } from "./error";
import {
  ExplainError,
  ErrorEnum,
  useErrorEnum,
  Infallible,
  SuppressUncaught,
  DiscloseUncaught,
  ErrorLoggerProvider,
} from "./error";
import {
  BrowserI18nProvider,
  createI18n,
  createI18nProvider,
  requestLocale,
} from "./i18n";
import { runtime } from "./jsx";
import { Breadcrumbs } from "./layout/Breadcrumbs";
import { BuildInfo } from "./layout/BuildInfo";
import { CanonicalLink } from "./layout/CanonicalLink";
import { HTMLDescription } from "./layout/HTMLDescription";
import { HTMLKeywords } from "./layout/HTMLKeywords";
import { HTMLTitle } from "./layout/HTMLTitle";
import { LangSwitcher } from "./layout/LangSwitcher";
import { LinkLoading } from "./layout/LinkLoading";
import { Loading } from "./layout/Loading";
import { NextPage } from "./layout/NextPage";
import { PageOutline } from "./layout/PageOutline";
import { PageTree } from "./layout/PageTree";
import { ProjectPicker } from "./layout/ProjectPicker";
import { RootPadding } from "./layout/RootPadding";
import { Search } from "./layout/Search";
import { SuggestEdit } from "./layout/SuggestEdit";
import { VersionPicker } from "./layout/VersionPicker";
import { ViewPageSource } from "./layout/ViewPageSource";
import { useColumnLayout } from "./layout/useColumnLayout";
import { onceLoader } from "./loaders";
import { PageRenderer } from "./page";
import { ScrollRestore } from "./page/anchoring";
import { breakpoint, themeProvider, useThemeToken } from "./theme";
import type { ThemeOverrides } from "./theme";

export type Environment = {
  pathFormat: PathFormat;
  repoLoader: RepoLoader;
  siteExtras?: SiteExtras;
  themeOverrides?: ThemeOverrides;
  externalRouter?: ExternalRouter;
};

/**
 * implementation of URL pathname parsing and formatting
 */
export type PathFormat = {
  /**
   * parse a path into a {@link MatchedPath} for page lookup
   */
  parse: (path: string) => MatchedPath | undefined;
  /**
   * reformat a {@link MatchedPath} into a canonical pathname which will be used for
   * linking and navigation
   */
  build: (data: MatchedPath) => string;
};

/**
 * variables that uniquely identify a specific page
 */
export type MatchedPath = Readonly<{
  /** repo name */
  repo: string;
  /** version, could be a PEP-440 tag or aliases such as "main" */
  ref?: string;
  /** language the page is in */
  lang?: string;
  /** the page's file path (the "docname" in Sphinx) */
  suffix?: string;
}>;

/**
 * function to fetch a specific version of docs given the {@link MatchedPath}
 *
 * @throws {ErrorEnum} — loaders MUST throw an {@link ErrorEnum} with
 * {@link ErrorEnum.cause cause} set to `"repo-4xx"` if it is confident that a requested
 * repo doesn't exist (e.g. as determined by a 404 {@link Response.status status}).
 */
export type RepoLoader = (req: {
  path: MatchedPath;
  /**
   * if the loader runs abortable operations such as {@link fetch} then it should
   * pass along the {@link AbortSignal}
   *
   * aborting could happen if e.g. user navigates to a 3rd page when the next page
   * hasn't finished loading yet
   */
  signal: AbortSignal;
  /**
   * the {@link Pep440Version} parsed from {@link MatchedPath.ref ref}, if it was parsable
   */
  pep440?: MaybePEP440;
  logger: ErrorLoggingImpl;
  prefetch: (result: Project[]) => Promise<void> | null;
}) => Promise<{
  fetched: Project[];
  pending: PartialProject[];
}>;

/**
 * extra metadata for specific repos
 *
 * keys correspond to {@link MatchedPath.repo}
 */
export type RepoExtras = {
  readonly [repo: string]: {
    /**
     * position of this project in the {@link ProjectPicker}
     */
    readonly displayOrder?: number;
    /**
     * display name of this project
     *
     * if not provided, the project use the (usually lowercase)
     * {@link MatchedPath.repo repo} for its name
     *
     * instead of a string, could also be an object, where keys are locale tags such
     * as `"zh-CN"` and values are the names in the corresponding languages
     */
    readonly projectName?: string | { [locale: string]: string };
    /**
     * preprocess chapter names before showing them in the {@link PageTree}
     *
     * this could be used to e.g. shorten titles that are too long
     */
    readonly chapterName?: (item: Readonly<SidebarItem>) => ReactNode;
    /**
     * return `true` if a given version of this project should be show in the
     * {@link VersionPicker}, and `false` otherwise
     *
     * by default, all versions are shown. use this to filter them.
     *
     * note that this does **not** prevent a version from being visited
     * (which is currently unsupported). this only hides them in the dropdown.
     * hidden versions will still be accessible using a direct link
     */
    readonly versionFilter?: (version: MaybePEP440) => boolean;
    /**
     * return an optional {@link MatchedPath redirect} for a given route
     *
     * function will receive the set of all routes from the prospective project
     * if available
     */
    readonly redirectPage?: (args: {
      ref?: string;
      lang?: string;
      suffix?: string;
      pep440?: MaybePEP440;
      routes?: Set<string>;
    }) => Partial<MatchedPath> | false | undefined;
  };
};

/**
 * extra customization options
 */
export type SiteExtras = Readonly<{
  /** @see {@link RepoExtras} */
  repos?: RepoExtras;
  /**
   * return the final section of the HTML `<title>`
   *
   * @see {@link HTMLTitle}
   */
  useSiteTitle?: (
    v: PageContent,
    s: RepoContent,
    m: {
      path: string[];
      items: SidebarItem[];
    } | null,
  ) => string;
  /**
   * return the final section of the HTML `<mate keywords=''>`
   *
   * @see {@link HTMLKeywords}
   */
  useSiteKeywords?: (v: PageContent, s: RepoContent) => string;
  /**
   * return the final section of the HTML `<mate description=''>`
   *
   * @see {@link HTMLDescription}
   */
  getSiteDescription?: (v: PageContent, s: RepoContent) => Promise<string>;
  /**
   * additional components to render into the UI, will receive metadata about the
   * current repo and page if available
   *
   * @see {@link SlottedComponent}
   */
  slots?: {
    /**
     * @see {@link PageEnd}
     */
    ArticleEnd?: SlottedComponent;
  };
  /**
   * extra routes to be added to the router
   */
  routes?: RouteObject[];
  /**
   * event handlers for logging purposes
   */
  logger?: ErrorLoggingImpl;
}>;

export type SlottedComponent = (props: {
  repo: Readonly<Pick<Project, "repo" | "ref" | "lang">>;
  page: { path: string; title?: string } | undefined;
  frontmatter?: Readonly<FrontMatter>;
}) => ReactNode;

/**
 * if the outer application is also a react-router app, states must be synchronized
 * between the two applications, or else navigation happening on either side may
 * not result in the correct UI updates in or outside the application
 */
export type ExternalRouter = {
  /**
   * subscribe to outer router changes
   *
   * this hook must update whenever the outer {@link useLocation} updates so that the
   * inner app can update its internal states
   */
  useExternalLocation: () => ExternalURL;
  /**
   * publish inner route changes to the outer router
   *
   *
   * this hook must return a {@link useNavigate} equivalent callback that the inner
   * app can invoke to update the outer router.
   */
  useExternalNavigate: () => (u: ExternalURL, options: { replace: true }) => void;
};

type ExternalURL = Pick<URL, "pathname" | "search" | "hash">;

export type PartialProject = Pick<Project, "repo"> & Partial<Pick<Project, "ref">>;

/**
 * @see {@link useSiteContent} which makes this data available to components
 */
type SiteContent = {
  path: PathFormat;
  projects: Project[];
  versions: VersionMap;
  extras: SiteExtras;
};

type LoaderData = {
  repo: RepoContent;
  page: PageContent;
};

/**
 * @see {@link useRepoContent} which makes this data available to components
 */
type RepoContent = {
  project: Project;
  message: RouterMessage;
};

type RouterMessage = {
  redirectedFrom?: {
    matched: MatchedPath;
    printed: string;
  };
};

/**
 * @see {@link usePageContent} and {@link usePartialPageContent} which makes
 * this data available to components
 */
type PageContent = {
  suffix: string;
  content: Awaited<ReturnType<ContentFunction>>;
};

declare module "./error" {
  interface ErrorContextImpl {
    ["router"]: Partial<Pick<LoaderData, "repo">>;
  }

  interface ErrorCauseImpl {
    ["page-404"]: {
      path: Partial<MatchedPath>;
    };
  }

  interface ErrorLoggingImpl {
    /** will be invoked when the router redirects a page to another route */
    onHttp307?: (from: MatchedPath, into: MatchedPath) => void;
    /** will be invoked when the router failed to match a repo or page */
    onHttp404?: (from: Partial<MatchedPath>) => void;
  }
}

export function createDocumentationSite(env: Environment) {
  const { pathFormat, themeOverrides, externalRouter, siteExtras = {} } = env;
  const { parse: parsePath } = pathFormat;
  const { logger = {} } = siteExtras;

  const repoLoader = onceLoader(env.repoLoader);

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
        refetchOnMount: false,
        retry: false,
      },
    },
  });

  const i18n = createI18n(tryLocaleFromPath(window.location.pathname).best.toString());
  const I18nProvider = createI18nProvider({ i18n, useCurrentLocale });

  const ThemeProvider = themeProvider(themeOverrides);

  const MemoizedSite = memo(DocumentationSite);
  const MemoizedPage = memo(DocumentationPage);

  const router = createRouter();

  return {
    DocumentationSite: MemoizedSite,
    DocumentationTheme,
    i18n,
  };

  function DocumentationSite() {
    return (
      <HelmetProvider>
        <QueryClientProvider client={queryClient}>
          <ErrorLoggerProvider {...logger}>
            <DocumentationTheme>
              <RouterProvider router={router} />
            </DocumentationTheme>
          </ErrorLoggerProvider>
        </QueryClientProvider>
      </HelmetProvider>
    );
  }

  function createRouter() {
    const routes200 = ":route-200";

    /** instances that are fully loaded */
    const fetched: Project[] = [];

    /**
     * all versions, including partially loaded projects, sorted by PEP-440
     *
     * @see {@link sortedVersions}
     */
    const versions: VersionMap = {};

    /**
     * repos and versions that are known but are not fetched yet
     *
     * they will appear in {@link ProjectPicker} and {@link VersionPicker},
     * and will be fetched upon navigation
     *
     * these are returned by the {@link RepoLoader loader}
     */
    const pending: PartialProject[] = [];

    /**
     * @see {@link derivePartialPaths}
     */
    const derived: Map<string, MatchedPath> = new Map();

    /**
     * @see {@link buildPath}
     */
    const redirections: Map<string, MatchedPath | false> = new Map();

    let inflight: Promise<unknown> = Promise.resolve();

    const hydrateFallbackElement = (
      <RootPadding>
        <Loading />
      </RootPadding>
    );

    const routes: RouteObject[] = [
      {
        element: (
          <SiteContentProvider>
            <Outlet />
            <SyncExternalRouter />
          </SiteContentProvider>
        ),
        errorElement: (
          <SiteContentProvider>
            <RootPadding>
              <ExplainError />
            </RootPadding>
            <SyncExternalRouter />
          </SiteContentProvider>
        ),
        children: [
          ...(siteExtras?.routes ?? []),
          {
            id: routes200,
            children: [],
          },
          {
            path: "/*",
            loader: redirectOrDoNothing,
            element: hydrateFallbackElement,
            hydrateFallbackElement,
          },
        ],
      },
    ];

    /**
     * à la <https://docs.djangoproject.com/en/5.2/ref/contrib/messages/>
     */
    const { takeMessage, sendMessage } = (() => {
      let state: RouterMessage | undefined = undefined;
      const sendMessage = (next: RouterMessage) => {
        state = next;
      };
      const takeMessage = () => {
        const data = state;
        state = undefined;
        return data;
      };
      return { takeMessage, sendMessage };
    })();

    return createBrowserRouter(routes, {
      patchRoutesOnNavigation: (req) => {
        const future = discoverContent(req);
        inflight = future
          .catch((err) => Promise.reject(ErrorEnum.catch({ code: "uncaught" }, err)))
          .finally(() => (inflight = Promise.resolve()));
        return future;
      },
    });

    /**
     * @see {@link DOMRouterOpts.patchRoutesOnNavigation patchRoutesOnNavigation}
     * @see https://reactrouter.com/6.30.1/routers/create-browser-router#optspatchroutesonnavigation
     * @see https://remix.run/blog/fog-of-war
     */
    async function discoverContent(req: PatchRoutesOnNavigationFunctionArgs) {
      const path = parsePath(req.path);

      if (path?.repo === undefined) {
        return;
      }

      const result = await repoLoader({
        path,
        pep440: path.ref ? maybePEP440(path.ref) : undefined,
        signal: req.signal,
        logger,
        prefetch: (fetched) => {
          fetched = fixLocaleTags(fetched);
          const suffix = path.suffix || ".";
          const target = getPartialProject(path, fetched)?.module.sitemap[suffix];
          if (target) {
            return target.exports().then(() => {});
          } else {
            return null;
          }
        },
      }).then(({ fetched, pending }) => {
        return { fetched: fixLocaleTags(fetched), pending };
      });

      if (!result.fetched.length && !result.pending.length) {
        return;
      }

      const routes = result.fetched //
        .map((project) => [
          ...Object.entries(project.module.sitemap) //
            .map(([suffix, page]) => deriveContentPage({ project, suffix, page })),
          derivePageNotFound({ project }),
        ])
        .flat();

      req.patch(routes200, routes);

      fetched.push(...uniqueProjects(result.fetched));
      pending.push(...uniqueProjects(result.pending));
      Object.assign(versions, sortedVersions([...fetched, ...pending]));

      derivePartialPaths();
    }

    function SiteContentProvider({ children }: PropsWithChildren) {
      return (
        <I18nProvider>
          <SiteContentContext.Provider
            value={{
              path: { parse: parsePath, build: buildPath },
              extras: siteExtras ?? {},
              projects: fetched,
              versions,
            }}
          >
            {children}
          </SiteContentContext.Provider>
        </I18nProvider>
      );
    }

    /**
     * {@link PathFormat.build build paths} with potential
     * {@link RepoExtras[string].redirectPage redirection}
     */
    function buildPath({ repo, ref, lang, suffix }: MatchedPath): string {
      const from = { repo, ref, lang, suffix };
      const key = stableHash(from);
      const memoized = redirections.get(key);
      if (memoized === false) {
        return pathFormat.build(from);
      } else if (memoized) {
        return pathFormat.build(memoized);
      }
      const redirectFn = siteExtras?.repos?.[repo]?.redirectPage;
      if (redirectFn) {
        const sitemap = getProject({ repo, ref, lang }, fetched)?.module.sitemap;
        const routes = sitemap ? new Set(Object.keys(sitemap)) : undefined;
        const pep440 = ref ? maybePEP440(ref) : undefined;
        const result = redirectFn({ ref, lang, suffix, routes, pep440 });
        if (result !== undefined) {
          const final = { ...from, ...result };
          redirections.set(key, final);
          return pathFormat.build(final);
        } else {
          return pathFormat.build(from);
        }
      } else {
        return pathFormat.build(from);
      }
    }

    /**
     * generate a {@link RouteObject route} for each page in the repo
     */
    function deriveContentPage({
      project,
      page,
      suffix,
    }: {
      project: Project;
      page: ProjectPage;
      suffix: string;
    }): RouteObject {
      return {
        path: buildPath({ ...project, suffix }),
        loader: async (): Promise<LoaderData> => {
          try {
            const { render } = await backoff(3, () => page.exports());
            const content = await render({ ...runtime, useMDXComponents });
            i18n.activate(project.lang, i18n.locales);
            const message = takeMessage() ?? {};
            return {
              repo: { project, message },
              page: { suffix, content },
            };
          } catch (err) {
            const message = takeMessage() ?? {};
            const ctx: ErrorContextImpl["router"] = {
              repo: { project, message },
            };
            throw ErrorEnum.catch({ code: "router", ...ctx }, err);
          }
        },
        element: <MemoizedPage />,
        errorElement: <MemoizedPage />,
        hydrateFallbackElement,
      };
    }

    /**
     * generate a catch all {@link RouteObject route} for paths to non-existent pages
     *
     * the loader will try to redirect
     */
    function derivePageNotFound({ project }: { project: Project }): RouteObject {
      return {
        path: buildPath({ ...project, suffix: "*" }),
        loader: async (args) => {
          await inflight;
          const tried = tryPaths(args);
          switch (tried.status) {
            case 307:
              logger?.onHttp307?.(tried.from, tried.into);
              return tried.response;
            case 404: {
              const message = takeMessage() ?? {};
              throw new ErrorEnum(
                { code: "router", repo: { project, message } },
                { code: "page-404", path: tried.matched ?? {} },
              );
            }
          }
        },
        element: <MemoizedPage />,
        errorElement: <MemoizedPage />,
        hydrateFallbackElement,
      };
    }

    /**
     * loader for when not even a repo was matched
     *
     * in most cases this is when a partial pathname was provided, in which case the
     * {@link tryPaths} function will try to complete the pathname using already loaded data
     * see {@link derivePartialPaths} for how partial matches are generated
     *
     * if a repo was not loaded, the {@link RepoLoader} should throw a 404 {@link ErrorEnum}
     * resulting in this loader never called. the `case 404:` code path is therefore
     * theoretically unreachable
     */
    async function redirectOrDoNothing(args: LoaderFunctionArgs) {
      await inflight;
      const tried = tryPaths(args);
      switch (tried.status) {
        case 307:
          logger?.onHttp307?.(tried.from, tried.into);
          return tried.response;
        case 404:
          return null;
      }
    }

    /**
     * in case of a route mismatch, try to find a route to fall back to. this is intended
     * to handle cases like:
     *
     * - fall back to a language when a language was not specified in the pathname
     *   or if docs for this repo isn't translated to the requested language
     * - fall back to a stable version when a version number was not specified in the
     *   pathname or if the version doesn't exist
     * - other cases provided by a repo's {@link RepoExtras[string].redirectPage redirectPage}
     *   function
     *
     * see {@link derivePartialPaths} for how partial matches are generated
     */
    function tryPaths({ request }: LoaderFunctionArgs) {
      const url = new URL(request.url, "https://example.org");
      const from = url.pathname;
      const matched = parsePath(from);
      if (!matched?.repo) {
        return {
          status: 404 as const,
          matched,
        };
      }
      let { repo, ref, lang, suffix } = matched;
      lang = requestLocale(lang).best.toString();
      const derived =
        getDerived({ repo, ref, lang, suffix }) ??
        getDerived({ repo, ref, lang }) ??
        getDerived({ repo, lang }) ??
        getDerived({ repo, ref }) ??
        getDerived({ repo }) ??
        matched;
      [ref, lang] = [derived.ref, derived.lang];
      if (derived.suffix) {
        suffix = derived.suffix;
      } else {
        suffix = matched.suffix;
      }
      const pathname = buildPath({ suffix, repo, ref, lang });
      if (from === pathname) {
        return {
          status: 404 as const,
          matched,
        };
      } else {
        sendMessage({ redirectedFrom: { matched, printed: from } });
        const referrer = parsePath(window.location.pathname);
        let final: string;
        if (referrer?.repo === matched.repo && referrer.suffix === matched.suffix) {
          const { search, hash } = window.location;
          final = routerBuildPath({ pathname, search, hash });
        } else {
          final = routerBuildPath({ pathname });
        }
        return {
          status: 307 as const,
          response: redirect(final, 307),
          from: { ...matched },
          into: { suffix, repo, ref, lang },
        };
      }
    }

    /**
     * generate partial path matches for {@link tryPaths} to fall back to
     */
    function derivePartialPaths() {
      const localized = getSupportedLanguages(i18n);

      for (const [repo, refs] of Object.entries(versions)) {
        for (const { raw: ref } of [...refs.tags, ...refs.head, ...refs.rest]) {
          const translated: string[] = [];
          const untranslated: (string | undefined)[] = [undefined];
          for (const lang of localized) {
            if (getProject({ repo, ref, lang }, fetched)) {
              translated.push(lang);
            } else {
              untranslated.push(lang);
            }
          }

          // derive noop redirects to known versions which will
          // result in a repo-level 404 page

          for (const lang of translated) {
            setDerived({ repo, ref, lang }, { repo, ref, lang });
          }

          // derive redirects for locales that don't exist in a particular version:

          const best = matchLocale(localized, translated, translated[0]);
          if (best) {
            for (const lang of untranslated) {
              setDerived({ repo, ref, lang }, { repo, ref, lang: best });
            }
          } else {
            // this is a PartialProject
          }
        }

        const sorted = [...refs.head, ...refs.tags];
        sorted.reverse();

        const stable = findStableVersion({ sorted })?.[1].raw;
        const latest = refs.head[0]?.raw;

        for (const lang of new Set([undefined, ...localized])) {
          // derive redirects to a stable version for paths without a version

          if (stable) {
            for (const ref of [undefined, "stable"]) {
              if (ref !== stable) {
                setDerived(
                  { repo, ref, lang },
                  { repo, ref: stable, lang: lang || i18n.locale },
                );
              }
            }
          }

          // derive redirects to HEAD for common aliases for HEAD

          if (latest) {
            for (const ref of ["main", "master", "latest", "HEAD"]) {
              if (ref !== latest) {
                setDerived(
                  { repo, ref, lang },
                  { repo, ref: latest, lang: lang || i18n.locale },
                );
              }
            }
          }
        }
      }

      // derive redirects to first page for repos without an index page

      for (const {
        repo,
        ref,
        lang,
        module: {
          sitemap: routes,
          manifest: { sidebar },
        },
      } of fetched) {
        const suffixes = Object.keys(routes);
        if (!suffixes.includes(".") && !suffixes.includes("")) {
          for (const { kind, key } of iterSidebar(sidebar)) {
            if (kind === "doc") {
              setDerived(
                { repo, ref, lang, suffix: "" },
                { repo, ref, lang, suffix: key },
              );
              break;
            }
          }
        }
      }
    }

    function getDerived(partial: MatchedPath): MatchedPath | undefined {
      const { repo, ref, lang, suffix } = partial;
      const key = stableHash({ repo, ref, lang, suffix });
      return derived.get(key);
    }

    function setDerived(partial: MatchedPath, complete: MatchedPath) {
      const { repo, ref, lang, suffix } = partial;
      const key = stableHash({ repo, ref, lang, suffix });
      derived.set(key, complete);
    }
  }

  /** the root layout */
  function DocumentationPage() {
    const { outer, left, center, right, resizeHandle, toolbar } = useColumnLayout();
    return (
      <ScrollRestore scrollable={() => document.documentElement}>
        <SuppressUncaught>{toolbar}</SuppressUncaught>
        <PanelGroup {...outer}>
          <Panel {...left}>
            <LeftSidebarOuter>
              <DiscloseUncaught
                explain={<Trans>Failed to display navigation menus</Trans>}
              >
                <Pickers>
                  <ProjectPicker />
                  <VersionPicker />
                  <SecondaryPickers>
                    <Search />
                    <LangSwitcher />
                  </SecondaryPickers>
                </Pickers>
              </DiscloseUncaught>
              <PickerDivider />
              <DiscloseUncaught
                explain={<Trans>Failed to display table of contents</Trans>}
              >
                <PageTree />
              </DiscloseUncaught>
            </LeftSidebarOuter>
          </Panel>
          {resizeHandle}
          <Panel {...center}>
            <ArticleOuter>
              <ArticleInner>
                <Infallible>
                  <SuppressUncaught>
                    <HTMLTitle />
                    <CanonicalLink />
                    <HTMLKeywords />
                    <HTMLDescription />
                    <Breadcrumbs />
                  </SuppressUncaught>
                </Infallible>
                <ExplainError />
                <Notifications>
                  <SuppressUncaught>
                    <BrowserI18nProvider>
                      <SuggestLanguage />
                      <NotTranslated />
                    </BrowserI18nProvider>
                    <IsNotebook />
                    <IsPrerelease />
                    <NoSuchVersion />
                    <RedirectedFrom />
                  </SuppressUncaught>
                </Notifications>
                <Infallible>
                  <DiscloseUncaught
                    explain={<Trans>Failed to display article content</Trans>}
                  >
                    <PageRenderer Link={IntraLink}>
                      <PageContent />
                    </PageRenderer>
                  </DiscloseUncaught>
                  <SuppressUncaught>
                    <PageEnd />
                    <NextPage />
                  </SuppressUncaught>
                </Infallible>
              </ArticleInner>
            </ArticleOuter>
          </Panel>
          {resizeHandle}
          <Panel {...right}>
            <RightSidebarOuter>
              <Infallible>
                <SuppressUncaught>
                  <StatusBar>
                    <BuildInfo />
                    <ViewPageSource />
                    <SuggestEdit />
                  </StatusBar>
                </SuppressUncaught>
                <DiscloseUncaught
                  explain={<Trans>Failed to display article outline</Trans>}
                >
                  <PageOutline />
                </DiscloseUncaught>
              </Infallible>
            </RightSidebarOuter>
          </Panel>
        </PanelGroup>
      </ScrollRestore>
    );
  }

  function PageContent() {
    const {
      page: {
        content: { default: Article },
      },
    } = usePageContent();
    return <Article />;
  }

  function DocumentationTheme({ children }: PropsWithChildren) {
    return (
      <ThemeProvider>
        <ThemeContextBridge>{children}</ThemeContextBridge>
      </ThemeProvider>
    );
  }

  function ThemeContextBridge({ children }: PropsWithChildren) {
    const tokens = useThemeToken();
    return (
      <ConfigProvider
        theme={{
          token: {
            fontFamily: tokens.fonts.sansSerif,
            fontFamilyCode: tokens.fonts.monospace,
          },
        }}
      >
        {children}
      </ConfigProvider>
    );
  }

  function tryLocaleFromPath(pathname: string) {
    return requestLocale(parsePath(pathname)?.lang);
  }

  function useCurrentLocale() {
    return tryLocaleFromPath(useLocation().pathname);
  }

  function SyncExternalRouter() {
    const { useExternalLocation = useLocation, useExternalNavigate = useNavigate } =
      externalRouter ?? {};

    const keyFn = ({ pathname, search, hash }: ExternalURL) =>
      stableHash({ pathname, search, hash });

    const timestamped = useTimestamped<ExternalURL>({ keyFn });

    const internal = timestamped(useLocation());
    const external = timestamped(useExternalLocation());

    const internalNavigate = useNavigate();
    const externalNavigate = useExternalNavigate();

    useEffect(() => {
      if (keyFn(internal.value) === keyFn(external.value)) {
        return;
      }
      if (internal.age > external.age) {
        internalNavigate(external.value, { replace: true });
      }
      if (external.age > internal.age) {
        externalNavigate(internal.value, { replace: true });
      }
    });

    return null;
  }

  function useTimestamped<T>({ keyFn }: { keyFn: (value: T) => string }) {
    const tracking = useRef<[number, T][]>([]).current;
    const now = Date.now();
    let index = 0;
    return (next: T) => {
      const [old, prev] = (tracking[index] ??= [now, next]);
      const prevKey = keyFn(prev);
      const nextKey = keyFn(next);
      tracking[index] = [now, next];
      index += 1;
      if (prevKey !== nextKey) {
        return { age: 0, value: next };
      } else {
        return { age: now - old, value: prev };
      }
    };
  }

  async function backoff<T>(n: number, fn: () => Promise<T>): Promise<T> {
    let error: unknown = undefined;
    for (let i = 0; i < n; i++) {
      try {
        return await fn();
      } catch (e) {
        error = e;
        console.warn(e);
        await new Promise((resolve) => setTimeout(resolve, Math.pow(2, i) * 1000));
      }
    }
    throw error;
  }
}

/**
 * @see {@link sortPEP440}
 */
function sortedVersions(versions: PartialProject[]): VersionMap {
  const unsorted: { [x: string]: string[] } = {};

  for (const { repo, ref } of versions) {
    const items = (unsorted[repo] ??= []);
    if (ref) {
      items.push(ref);
    }
  }

  const sorted: VersionMap = {};

  Object.entries(unsorted).forEach(([project, versions]) => {
    sorted[project] = sortPEP440([...new Set(versions)]);
  });

  return sorted;
}

type VersionMap = { [x: string]: ReturnType<typeof sortPEP440> };

export type MaybePEP440 = {
  version: Pep440Version | null;
  label: WellKnownLabel | null;
  raw: string;
};

type WellKnownLabel = keyof typeof wellKnownLabels;

const wellKnownLabels = {
  stable: 6,
  rc: 5,
  beta: 4,
  alpha: 3,
  dev: 2,
  head: 1,
};

export function maybePEP440(raw: string): MaybePEP440 {
  const version = parsePEP440(raw);
  if (version !== null) {
    let label: WellKnownLabel | null = null;
    if (version.is_devrelease) {
      label = "dev";
    } else if (!version.is_prerelease) {
      label = "stable";
    } else {
      const maybe = /(rc|b|a)[a-z]*(\d+)/i.exec(version.pre.join(""))?.[1] ?? null;
      if (maybe === "a") {
        label = "alpha";
      }
      if (maybe === "b") {
        label = "beta";
      } else {
        label = null;
      }
    }
    return { raw, version, label };
  } else if (raw === "main" || raw === "master" || raw === "latest" || raw === "HEAD") {
    return { raw, version, label: "head" };
  } else if (raw === "stable") {
    return { raw, version, label: "stable" };
  } else {
    return { raw, version, label: null };
  }
}

const parsePEP440: typeof explainPEP440 = (() => {
  const memo: Map<string, ReturnType<typeof explainPEP440>> = new Map();
  return (version) => {
    if (version === null || version === undefined) {
      return null;
    } else {
      const parsed = explainPEP440(version);
      memo.set(version, parsed);
      return parsed;
    }
  };
})();

/**
 * sort a list of versions using their potential {@link Pep440Version} ordering
 * and collect them into buckets base on whether they are a well-formed {@link Pep440Version}
 * or other aliases such as `HEAD` or `main`
 */
function sortPEP440(items: string[]): {
  tags: readonly MaybePEP440[];
  head: readonly MaybePEP440[];
  rest: readonly MaybePEP440[];
} {
  const tags: MaybePEP440[] = [];
  const rest: MaybePEP440[] = [];
  const head: MaybePEP440[] = [];

  for (const raw of items) {
    const version = maybePEP440(raw);
    switch (version.label) {
      case "head":
        head.push(version);
        break;
      case null:
        rest.push(version);
        break;
      default:
        tags.push(version);
        break;
    }
  }

  tags.sort((a, b) => comparePEP440(a.raw, b.raw));
  rest.sort((a, b) => a.raw.localeCompare(b.raw));

  return { tags, head, rest };
}

const SiteContentContext = createContext<SiteContent>({
  path: {
    parse: () => ({ repo: "", ref: undefined, lang: undefined, "*": undefined }),
    build: () => "",
  },
  projects: [],
  versions: {},
  extras: {},
});

export function useSiteContent() {
  return useContext(SiteContentContext);
}

export function useRepoContent(): Pick<LoaderData, "repo"> {
  const value = useLoaderData<Partial<LoaderData> | null>();
  const error = useErrorEnum();
  if (value?.repo) {
    const { repo } = value;
    return { repo };
  } else if (error?.context.code === "router" && error.context.repo) {
    const { repo } = error.context;
    return { repo };
  } else {
    throw error;
  }
}

export function usePageContent(): Pick<LoaderData, "page"> {
  const value = usePartialPageContent();
  const error = useErrorEnum();
  if (value.page) {
    return { page: value.page };
  } else {
    throw error;
  }
}

export function usePartialPageContent(): Partial<Pick<LoaderData, "page">> {
  return useLoaderData<Partial<LoaderData> | null>() ?? {};
}

export function useFullContent(): LoaderData {
  return {
    ...useRepoContent(),
    ...usePageContent(),
  };
}

export function findStableVersion({
  sorted,
}: {
  sorted: readonly MaybePEP440[];
}): [number, MaybePEP440] | undefined {
  let target: [number, MaybePEP440] | undefined = undefined;
  let weight: number = -Infinity;
  for (let i = 0; i < sorted.length; i++) {
    const tag = sorted[i];
    if (!tag.label) {
      continue;
    }
    const w = wellKnownLabels[tag.label];
    if (w > weight) {
      target = [i, tag];
      weight = w;
    }
  }
  return target;
}

export function useProjectLanguages({ repo, ref }: Pick<Project, "repo" | "ref">) {
  const { projects } = useSiteContent();
  return projects
    .filter((project) => project.repo === repo && project.ref === ref)
    .map((project) => project.lang);
}

export function getSupportedLanguages(i18n: I18n) {
  if (!i18n.locales) {
    return [];
  } else if (typeof i18n.locales === "string") {
    return [i18n.locales];
  } else {
    return i18n.locales;
  }
}

export function getSidebar(project: Project) {
  let sidebar = project.module.manifest.sidebar;
  if (sidebar.length === 1 && sidebar[0].children?.length) {
    sidebar = sidebar[0].children;
  }
  return sidebar;
}

export function getBreadcrumbs(
  sidebar: Sidebar,
  key: string | undefined,
): {
  path: string[];
  items: SidebarItem[];
} | null {
  if (key === undefined) {
    return null;
  }
  for (const item of sidebar) {
    if (item.children) {
      const found = getBreadcrumbs(item.children, key);
      if (found) {
        return {
          path: [item.key, ...found.path],
          items: [item, ...found.items],
        };
      }
    }
    if (item.key === key) {
      return { path: [item.key], items: [item] };
    }
  }
  return null;
}

export function* iterSidebar(sidebar: Sidebar): Generator<SidebarItem> {
  for (const item of sidebar) {
    yield item;
    if (item.children) {
      yield* iterSidebar(item.children);
    }
  }
}

/**
 * links in documentation content will render using this component
 *
 * most importantly, this renders "intra links" using {@link NavLink} instead of `<a>`
 * to support client-side navigation.
 *
 * "intra links" are links to other pages or other repos (anything parsable by
 * {@link PathFormat.parse})
 */
export function IntraLink({ to, ...props }: ComponentProps<typeof NavLink>) {
  const {
    path: { parse, build },
  } = useSiteContent();
  try {
    if (typeof to === "string") {
      const url = new URL(to);
      if (url.origin !== window.location.origin) {
        return <NavLink to={to} target="_blank" {...props} />;
      }
    }
  } catch {
    // not a url
  }

  const {
    pathname = "/",
    search,
    hash,
  } = typeof to === "string" ? routerParsePath(to) : to;

  const matched = parse(pathname);

  if (!matched?.repo || !matched.lang) {
    let { className, style, children, ...rest } = props;
    const renderProps = { isActive: false, isPending: false, isTransitioning: false };
    if (typeof className === "function") {
      className = className(renderProps);
    }
    if (typeof style === "function") {
      style = style(renderProps);
    }
    if (typeof children === "function") {
      children = children(renderProps);
    }
    // give up
    const href = typeof to === "string" ? to : routerBuildPath(to);
    return (
      <a href={href} className={className} style={style} {...rest}>
        {children}
      </a>
    );
  }

  const { repo, ref, lang, suffix = "" } = matched;

  return (
    <span>
      <NavLink
        to={routerBuildPath({
          pathname: build({ repo, ref, lang, suffix }),
          search,
          hash,
        })}
        {...props}
      />
      <LinkLoading
        repo={repo}
        ref_={ref}
        lang={lang}
        suffix={suffix}
        search={search}
        hash={hash}
        style={{ position: "relative", top: 2 }}
      />
    </span>
  );
}

/**
 * information about the location that is currently being navigated to, with
 * pathname parsed into {@link MatchedPath}
 *
 * this is used to show a spinner next to a clicked link, for example
 */
export function usePageNavigation():
  | {
      pathname: Partial<MatchedPath>;
      search: string | undefined;
      hash: string | undefined;
    }
  | undefined {
  const current = useLocation();

  const { state, location: opening } = useNavigation();

  const {
    path: { parse },
  } = useSiteContent();

  type Mutable<T> = {
    -readonly [P in keyof T]: T[P];
  };

  const changed: Mutable<Partial<MatchedPath>> = {};

  if (state === "idle" || opening === undefined) {
    return undefined;
  }

  const from = parse(current.pathname);
  const into = parse(opening.pathname);

  if (from?.repo !== into?.repo) {
    changed.repo = into?.repo;
  }
  if (from?.ref !== into?.ref) {
    changed.ref = into?.ref;
  }
  if (from?.lang !== into?.lang) {
    changed.lang = into?.lang;
  }
  if (from?.suffix !== into?.suffix) {
    changed.suffix = into?.suffix;
  }

  const { search, hash } = opening;

  return { pathname: changed, search, hash };
}

export function usePathPatcher(): (r: Pick<MatchedPath, "ref" | "lang">) => string {
  const { pathname, search, hash } = useLocation();
  const {
    path: { build, parse },
  } = useSiteContent();
  const match = parse(pathname);
  if (!match) {
    return () => "#";
  }
  const { repo, suffix, ...prev } = match;
  return ({ ref, lang }) => {
    ref ??= prev.ref;
    lang ??= prev.lang;
    const pathname = build({ repo, ref, lang, suffix });
    return routerBuildPath({ pathname, search, hash });
  };
}

type ResolvedRepoExtras = {
  projectName: string;
} & Required<Omit<RepoExtras[string], "projectName">>;

export function useRepoExtras(
  project: Pick<Project, "repo" | "lang">,
): ResolvedRepoExtras {
  const { extras } = useSiteContent();
  return getRepoExtras(project, extras);
}

export function getRepoExtras(
  project: Pick<Project, "repo" | "lang">,
  extras: SiteExtras | undefined,
): ResolvedRepoExtras {
  let {
    displayOrder = Infinity,
    projectName = project.repo,
    chapterName = (x: Readonly<SidebarItem>) => x.title,
    versionFilter = () => true,
    redirectPage = () => undefined,
  } = extras?.repos?.[project.repo] ?? {};
  if (typeof projectName !== "string") {
    projectName = projectName[project.lang];
  }
  return {
    displayOrder,
    projectName,
    chapterName,
    versionFilter,
    redirectPage,
  };
}

/**
 * measure window sizes for layout
 */
export function useClientMeasurements() {
  const {
    dimensions: { mobileWidth },
  } = useThemeToken();
  const [clientWidth, setClientWidth] = useState(document.documentElement.clientWidth);
  useEffect(() => {
    const listener = debounce(
      () => setClientWidth(document.documentElement.clientWidth),
      8,
      { trailing: true },
    );
    window.addEventListener("resize", listener);
    return () => window.removeEventListener("resize", listener);
  }, []);
  const smallScreen = clientWidth < mobileWidth;
  return { smallScreen, clientWidth };
}

function PageEnd() {
  const { extras: { slots: { ArticleEnd } = {} } = {} } = useSiteContent();
  const {
    repo: { project },
    page: { suffix, content },
  } = useFullContent();
  const title = usePageTitle();
  if (ArticleEnd) {
    const { repo, ref, lang } = project;
    return (
      <ArticleEnd
        repo={{ repo, ref, lang }}
        page={suffix ? { path: suffix, title } : undefined}
        frontmatter={content?.frontmatter}
      />
    );
  } else {
    return null;
  }
}

function usePageTitle() {
  const {
    repo: { project },
    page: { suffix },
  } = useFullContent();
  for (const item of iterSidebar(getSidebar(project))) {
    if (item.key === suffix) {
      return item.title;
    }
  }
  return undefined;
}

const LeftSidebarOuter = styled.nav`
  display: flex;
  flex-flow: column nowrap;
  gap: 8px;
  padding: 8px;

  .ant-select-selection-item,
  .ant-select-item-option-content {
    font-size: 16px;
    font-weight: 500;
  }

  ${"container-type"}: inline-size;
`;

const Pickers = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 8px;
  padding: 0;
`;

const PickerDivider = styled(Divider)`
  width: unset;
  min-width: unset;
  margin: 0;
`;

const SecondaryPickers = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
`;

const ArticleOuter = styled.div`
  position: relative;
  min-height: 100%;
  border-inline-start: 1px solid rgb(239 239 239);

  ${breakpoint("mobileWidth")} {
    border-inline-start: none;
  }
`;

const ArticleInner = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;
  width: 100%;
  min-width: 0;
  max-width: 1024px;
  padding: 1rem 2rem 2.5rem;
  margin: 0 auto;

  ${breakpoint("mobileWidth")} {
    min-width: 100vw;
    padding: 1rem 1rem 2rem;
  }

  img {
    max-width: 100% !important;
  }
`;

const RightSidebarOuter = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 12px;
  min-height: 0;
  padding: 16px;
  overflow: auto;

  ${"container-type"}: inline-size;
`;

const StatusBar = styled.aside`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.3rem;
  min-width: 0;
`;

/** @private these types are only referenced in docstrings */
export type __JSDoc = DOMRouterOpts | ErrorCause;
