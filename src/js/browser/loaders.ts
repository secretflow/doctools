import { matchPath } from "react-router";
import { stableHash } from "stable-hash";
import { z } from "zod";

import type { Sitemap } from "../docs/types";

import type {
  MaybePEP440,
  PartialProject,
  PathFormat,
  RepoLoader,
  MatchedPath,
} from "./app";
import { maybePEP440 } from "./app";
import type { ErrorContext } from "./error";
import { ErrorEnum } from "./error";
import { isSpuriousLocale } from "./i18n";

/**
 * return a URL that can be used to {@link fetch} content given an {@link NPMRequest}
 */
export type NPMProvider = (req: NPMRequest) => URL;

export type NPMRequest = {
  /**
   * {@link MatchedPath.repo project name}, note that is not the npm package name, and
   * {@link NPMLoaderOptions.providers providers} are expected to complete it
   */
  repo: string;
  file: string;
  version: NPMVersion;
};

export type NPMVersion =
  | {
      type: "tagged";
      pep440: MaybePEP440;
    }
  | {
      type: "canonical";
      version: string;
    }
  | {
      type: "latest";
    };

declare module "./error" {
  interface ErrorContextImpl {
    ["loader"]: unknown;
  }

  interface ErrorCauseImpl {
    ["repo-4xx"]: {
      repo: string;
      res: Response;
    };
  }

  interface ErrorLoggingImpl {
    onMirror200?: (mirror: string) => void;
    onMirror503?: (...mirrors: string[]) => void;
  }
}

type RepoLoaderArgs = Parameters<RepoLoader>[0];

export type NPMLoaderOptions = {
  providers: { [name: string]: NPMProvider };
};

/**
 * create a {@link RepoLoader} that loads content by fetching npm packages
 *
 * accept multiple {@link NPMLoaderOptions.providers providers}, in which case the loader
 * will conduct a speedtest the first time a request is made.
 */
export function npmLoader({ providers }: NPMLoaderOptions): RepoLoader {
  const LOADER_ERR: ErrorContext = { code: "loader" };

  const fetcher = (() => {
    let fastest: NPMProvider | undefined = undefined;

    return speedtest;

    async function speedtest({ req, src }: { req: NPMRequest; src: RepoLoaderArgs }) {
      const { signal } = src;

      if (fastest) {
        const mirror = fastest.name;
        try {
          return await request({ reg: fastest, req, opt: { signal } });
        } catch (e) {
          const { err, http4xx } = catch4xx(e, req.repo);
          if (!http4xx) {
            src.logger.onMirror503?.(mirror);
            fastest = undefined;
            return await speedtest({ req, src });
          } else {
            throw err;
          }
        }
      }

      const futures = Object.values(providers).map((reg) => {
        const abort = new AbortController();
        signal.addEventListener("abort", () => abort.abort(signal.reason));

        const future = (async () => {
          try {
            const res = await request({
              reg,
              req,
              opt: {
                cache: "reload",
                signal: abort.signal,
              },
            });
            await src.prefetch(res.mod);
            return { ...res, reg };
          } catch (e) {
            const { err, http4xx } = catch4xx(e, req.repo);
            if (http4xx) {
              futures.forEach((f) => f.abort.abort("received http 4xx"));
            }
            throw err;
          }
        })();

        return { future, abort };
      });

      try {
        const fulfilled = await Promise.any(futures.map((f) => f.future));
        if (fastest === undefined) {
          fastest = fulfilled.reg;
          src.logger.onMirror200?.(fastest.name);
        }
        return fulfilled;
      } catch (e) {
        fastest = undefined;
        src.logger.onMirror503?.(...Object.keys(providers));
        throw catch4xx(e, req.repo).err;
      } finally {
        futures.forEach((f) => f.abort.abort("requests will settle"));
      }
    }

    async function request({
      reg,
      req,
      opt,
    }: {
      reg: NPMProvider;
      req: NPMRequest;
      opt: RequestInit;
    }) {
      const abort1 = new AbortController();
      const abort2 = new AbortController();

      opt.signal?.addEventListener("abort", () => abort1.abort(opt.signal?.reason));
      opt.signal?.addEventListener("abort", () => abort2.abort(opt.signal?.reason));

      const response = await fetch(reg({ ...req }), {
        ...opt,
        signal: abort1.signal,
        redirect: "follow",
        priority: "high",
      }) //
        .then((res) => ErrorEnum.check(LOADER_ERR, res));

      const pkg = packageJsonV1.parse(await response.json());

      const {
        version,
        exports: {
          ".": { import: file },
        },
      } = pkg;

      const url = reg({
        repo: req.repo,
        version: { type: "canonical", version },
        file,
      }).toString();

      const mod = await fetch(url, { ...opt, signal: abort2.signal })
        .then((res) => ErrorEnum.check(LOADER_ERR, res))
        .then(() => import(/* @vite-ignore */ /* webpackIgnore: true */ url))
        .then((res: { default: Sitemap }) => ErrorEnum.check(LOADER_ERR, res))
        .then(({ default: sitemap }) => Object.values(sitemap));

      return { reg, pkg, mod };
    }
  })();

  return async function (src) {
    const {
      path: { repo },
      pep440,
    } = src;

    /**
     * the `latest` package will supply the version list of the requested repo
     *
     * @see {@link packageJsonV1}
     */
    const r1 = fetcher({
      req: {
        repo,
        file: "package.json",
        version: { type: "latest" },
      },
      src,
    });

    /**
     * the `tagged` package will be the actual requested version.
     *
     * note that there may not be a requested version: if a valid version number cannot
     * be parsed from the pathname then the application falls back to presenting the
     * most recent stable version (a redirection will occur).
     */
    const r2 =
      pep440 !== undefined
        ? fetcher({
            req: {
              repo,
              file: "package.json",
              version: { type: "tagged", pep440 },
            },
            src,
          })
        : r1;

    let [latest, target] = await Promise.allSettled([r1, r2]);

    if (target.status === "rejected") {
      if (pep440 !== undefined) {
        if (catch4xx(target.reason, repo).http4xx) {
          target = latest;
        }
      }
    }

    if (target.status === "rejected") {
      throw target.reason;
    }

    if (latest.status === "rejected") {
      latest = target;
    }

    const fetched = target.value.mod;

    const pending: PartialProject[] = [];

    const {
      value: {
        pkg: { "x-secretflow-refs": refs = [] },
      },
    } = latest;
    for (const version of refs) {
      pending.push({ repo, ref: version });
    }

    return { fetched, pending };
  };

  function catch4xx(err: unknown, repo: string) {
    let errors: ErrorEnum[];
    if (err instanceof AggregateError) {
      errors = err.errors.map((err) => ErrorEnum.catch(LOADER_ERR, err));
    } else {
      errors = [ErrorEnum.catch(LOADER_ERR, err)];
    }
    for (const err of errors) {
      const { cause: reason } = err;
      if (reason.code === "repo-4xx") {
        return { err, http4xx: true };
      }
      if (
        reason.code === "http" &&
        reason.res.status >= 400 &&
        reason.res.status < 500
      ) {
        const { res } = reason;
        const err = new ErrorEnum(LOADER_ERR, { code: "repo-4xx", repo, res });
        return { err, http4xx: true };
      }
    }
    return { err: errors[0], http4xx: false };
  }
}

const packageJsonV1 = z.object({
  ["version"]: z.string(),
  ["exports"]: z.object({
    ["."]: z.object({
      import: z.string(),
    }),
  }),
  ["x-secretflow-refs"]: z.array(z.string()).optional(),
});

export function onceLoader(loader: RepoLoader): RepoLoader {
  const seen = new Set<string>();
  return async (req) => {
    const {
      path: { repo, ref, lang, suffix },
    } = req;
    const key = stableHash({ repo, ref, lang, suffix });
    if (seen.has(key)) {
      return { fetched: [], pending: [] };
    }
    const result = await loader(req);
    seen.add(key);
    return result;
  };
}

export type FuzzyPathOptions = {
  parse: string[];
  check?: {
    ref?: (args: { repo: string; ref: string }) => boolean;
  };
} & Pick<PathFormat, "build">;

export function fuzzyPath({
  parse: formats,
  check,
  build,
}: FuzzyPathOptions): PathFormat {
  type ParsedPath = NonNullable<ReturnType<PathFormat["parse"]>>;

  const parse: PathFormat["parse"] = (path) => {
    let scored = -Infinity;
    let parsed: ParsedPath | undefined = undefined;
    for (const format of formats) {
      let { repo, ref, lang, "*": suffix } = matchPath(format, path)?.params ?? {};
      if (!repo) {
        continue;
      }
      if (lang && isSpuriousLocale(lang)) {
        lang = undefined;
      }
      if (ref && !maybePEP440(ref).label && !check?.ref?.({ repo, ref })) {
        ref = undefined;
      }
      const score = specificity({ repo, ref, lang, suffix });
      if (score >= scored) {
        parsed = { repo, ref, lang, suffix };
        scored = score;
      }
    }
    return parsed;
  };

  return { parse, build };

  function specificity(parsed: Partial<ParsedPath>) {
    let score = 0;
    const { repo, ref, lang } = parsed;
    if (repo) {
      score += 1;
    }
    if (ref) {
      score += 1;
    }
    if (lang) {
      score += 1;
    }
    return score as 0 | 1 | 2 | 3;
  }
}

/** @private these types are only referenced in docstrings */
export type __JSDoc = MatchedPath;
