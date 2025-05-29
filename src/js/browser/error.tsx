import { Trans } from "@lingui/react/macro";
import {
  Component,
  createContext,
  Fragment,
  memo,
  useCallback,
  useContext,
  useEffect,
  useRef,
} from "react";
import type { PropsWithChildren, ReactNode } from "react";
import { useRouteError } from "react-router";

import { ErrorDetails } from "./banners/ErrorDetails";
import { PageNotFound } from "./banners/PageNotFound";
import { RepoNotFound } from "./banners/RepoNotFound";
import { UncaughtRouteError } from "./banners/UncaughtRouteError";

export class ErrorEnum extends Error {
  override readonly cause: ErrorCause;
  public readonly context: ErrorContext;

  constructor(ctx: ErrorContext, err: ErrorCause) {
    super();
    this.context = ctx;
    this.cause = err;
  }

  static check<T extends Error>(ctx: ErrorContext, err: T): never;
  static check<T>(ctx: ErrorContext, err: T): T;
  static check<T>(ctx: ErrorContext, err: T) {
    if (err instanceof this) {
      if (err.context.code === "uncaught") {
        throw new this(ctx, err.cause);
      } else {
        throw err;
      }
    } else if (err instanceof Response) {
      if (err.status >= 400) {
        throw new this(ctx, { code: "http", res: err });
      } else {
        return err;
      }
    } else if (err instanceof Error) {
      throw new this(ctx, { code: "unknown", err });
    } else {
      return err;
    }
  }

  static catch(ctx: ErrorContext, err: ErrorContext): void;
  static catch(ctx: ErrorContext, err: unknown): ErrorEnum;
  static catch(ctx: ErrorContext, err: unknown): ErrorEnum {
    try {
      this.check(ctx, err);
      throw new this(ctx, {
        code: "unknown",
        err: err instanceof Error ? err : new Error(String(err)),
      });
    } catch (e) {
      return e as ErrorEnum;
    }
  }

  override get message(): string {
    switch (this.cause.code) {
      case "http":
      case "repo-4xx": {
        const { status, statusText, redirected, url } = this.cause.res ?? {};
        const headers = Object.fromEntries(this.cause.res?.headers.entries() ?? []);
        const data = { status, statusText, headers, redirected, url };
        return `Response ${JSON.stringify(data, null, 2)}`;
      }
      case "page-404":
        return `no such page ${JSON.stringify(this.cause.path)}`;
      case "unknown":
        return this.cause.err.stack || String(this.cause.err);
      default:
        try {
          return JSON.stringify(this.cause);
        } catch {
          return String(this);
        }
    }
  }

  override toString() {
    return this.message;
  }
}

export interface ErrorCauseImpl {
  ["http"]: { res: Response };
  ["unknown"]: { err: Error };
}

export interface ErrorContextImpl {
  ["uncaught"]: unknown;
}

export type ErrorCause = Readonly<TaggedUnion<ErrorCauseImpl>>;

export type ErrorContext = Readonly<TaggedUnion<ErrorContextImpl>>;

type TaggedUnion<T> = keyof T extends infer K
  ? K extends keyof T
    ? { code: K } & T[K]
    : never
  : never;

export function useErrorEnum(err?: unknown) {
  const error = useRouteError() ?? err ?? null;
  if (error === null) {
    return null;
  } else {
    return ErrorEnum.catch({ code: "uncaught" }, error);
  }
}

export function ExplainError({ error: err }: { error?: unknown }): ReactNode {
  const error = useErrorEnum(err);
  return (
    <Fragment>
      {(() => {
        switch (error?.context.code) {
          case "loader":
            switch (error.cause.code) {
              case "repo-4xx":
                return <RepoNotFound reason={error.cause} />;
              default:
                return <UncaughtRouteError />;
            }
          case "router":
            switch (error.cause.code) {
              case "page-404":
                return <PageNotFound reason={error.cause} />;
              default:
                return <UncaughtRouteError />;
            }
          case "uncaught":
            return <UncaughtRouteError />;
          case undefined:
            return null;
        }
      })()}
      <LogError error={err} />
    </Fragment>
  );
}

export function Infallible({ children }: PropsWithChildren) {
  if (useRouteError()) {
    return null;
  } else {
    return children;
  }
}

type UncaughtState = { error?: unknown };

export class SuppressUncaught extends Component<PropsWithChildren, UncaughtState> {
  constructor(props: PropsWithChildren) {
    super(props);
    this.state = {};
  }

  static getDerivedStateFromError(error: unknown): UncaughtState {
    return { error };
  }

  override componentDidCatch(error: Error): void {
    didCatchError(error);
  }

  override render(): ReactNode {
    const { error } = this.state;
    if (error) {
      return <LogError error={error} />;
    } else {
      return this.props.children;
    }
  }
}

export class DiscloseUncaught extends Component<
  PropsWithChildren<{ explain: ReactNode }>,
  UncaughtState
> {
  constructor(props: PropsWithChildren<{ explain: ReactNode }>) {
    super(props);
    this.state = {};
  }

  static getDerivedStateFromError(error: unknown): UncaughtState {
    return { error };
  }

  override componentDidCatch(error: Error): void {
    didCatchError(error);
  }

  override render(): ReactNode {
    const { error } = this.state;
    const { explain, children } = this.props;
    if (error) {
      return (
        <Fragment>
          <ErrorDetails type="error" title={<Trans>Page error</Trans>} error={error}>
            {explain}
          </ErrorDetails>
          <LogError error={error} />
        </Fragment>
      );
    } else {
      return children;
    }
  }
}

function didCatchError(error: Error) {
  const caught = ErrorEnum.catch({ code: "uncaught" }, error);
  switch (caught.context.code) {
    case "uncaught":
      break;
    default:
      throw error;
  }
}

export interface ErrorLoggingImpl {
  onHttp500?: (error: ErrorEnum) => void;
}

const ErrorLoggingContext = createContext<{
  log: (err: ErrorEnum) => void;
}>({
  log: () => {},
});

export function ErrorLoggerProvider({
  children,
  ...impl
}: PropsWithChildren<ErrorLoggingImpl>) {
  const debounced = useRef<ErrorEnum>();

  const log = useCallback(
    (error: ErrorEnum) => {
      if (error === debounced.current) {
        return;
      } else {
        debounced.current = error;
      }
      switch (error.cause.code) {
        case "unknown":
          impl.onHttp500?.(error);
          break;
        case "page-404":
          impl.onHttp404?.(error.cause.path);
          break;
        case "repo-4xx":
          impl.onHttp404?.(error.cause);
          break;
      }
    },
    [impl],
  );

  return (
    <ErrorLoggingContext.Provider value={{ log }}>
      {children}
    </ErrorLoggingContext.Provider>
  );
}

const LogError = memo(function ErrorLogger({ error: err }: { error?: unknown }) {
  const error = useErrorEnum(err);
  const { log } = useContext(ErrorLoggingContext);
  useEffect(() => {
    if (error) {
      log(error);
    }
  }, [error, log]);
  return null;
});

export function quoteUrl(text: string) {
  try {
    return JSON.stringify(decodeURIComponent(text));
  } catch {
    return JSON.stringify(text);
  }
}
