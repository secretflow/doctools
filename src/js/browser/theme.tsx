import { merge } from "lodash";
import { useContext } from "react";
import type { PropsWithChildren } from "react";
import { createGlobalStyle, ThemeProvider, ThemeContext } from "styled-components";
import type { StyleFunction, DefaultTheme } from "styled-components";

type Theme = {
  dimensions: {
    navbarHeight: number;
    scrollOffset: number;
    mobileWidth: number;
    mobileToolbarHeight: number;
  };
  spacing: {
    xs: string;
    s: string;
    sm: string;
    m: string;
    ml: string;
  };
  fonts: {
    sansSerif: string;
    monospace: string;
  };
  colors: {
    bg: {
      default: string;
      container: string;
      info: string;
      warning: string;
      highlight: string;
    };
    fg: {
      default: string;
      muted: string;
      strong: string;
      link: string;
      inverted: string;
      container: string;
    };
    blue: string;
    green: string;
    red: string;
    yellow: string;
    magenta: string;
    cyan: string;
    neutral: string;
  };
};

export type ThemeOverrides = PartialTheme<Theme>;

const { themeProvider, useThemeToken, theme, fromDefaultTheme } = createTheme<Theme>({
  dimensions: {
    navbarHeight: 36,
    scrollOffset: 16,
    mobileWidth: 1024,
    mobileToolbarHeight: 50,
  },
  spacing: {
    xs: "8px",
    s: "12px",
    sm: "16px",
    m: "20px",
    ml: "30px",
  },
  fonts: {
    sansSerif:
      "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji'",
    monospace:
      "ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace",
  },
  colors: {
    bg: {
      default: "#fdfdfe",
      container: "#f6f6f6",
      info: "#eef9fd",
      warning: "#fff8e6",
      highlight: "#fbe54e",
    },
    fg: {
      default: "rgb(0 0 0 / 88%)",
      muted: "#4c525b",
      strong: "#e83e8c",
      link: "#0060e6",
      inverted: "#fdfdfe",
      container: "#eaeaea",
    },
    blue: "#61afef",
    red: "#e06c75",
    green: "#98c379",
    yellow: "#e5c07b",
    magenta: "#c678dd",
    cyan: "#56b6c2",
    neutral: "#a0a0a0",
  },
});

export { themeProvider, useThemeToken, theme };

export function breakpoint(dim: keyof Theme["dimensions"]): StyleFunction<object> {
  return ({ theme }) => {
    const { dimensions } = fromDefaultTheme(theme);
    return `@media screen and (width < ${dimensions[dim]}px)`;
  };
}

function createTheme<T extends CSSProperties>(theme: T) {
  const brand = Symbol("theme");

  return {
    theme: intoSelectors(theme),
    themeProvider,
    useThemeToken,
    fromDefaultTheme,
  };

  function themeProvider(overrides?: PartialTheme<T>) {
    const final = merge({}, theme, overrides);

    const Style = createGlobalStyle`
      :root {
        ${intoVariables(final)
          .map(([k, v]) => `${k}: ${v};`)
          .join("\n")}
      }
    `;

    return ThemeProvider2;

    function ThemeProvider2({ children }: PropsWithChildren) {
      return (
        <ThemeProvider theme={{ [brand]: final }}>
          <Style />
          {children}
        </ThemeProvider>
      );
    }
  }

  function useThemeToken() {
    return fromDefaultTheme(useContext(ThemeContext));
  }

  function fromDefaultTheme(ctx: DefaultTheme | undefined) {
    return (ctx as { [brand]?: T } | undefined)?.[brand] ?? theme;
  }

  function intoSelectors<T extends CSSProperties>(
    tokens: T,
    prefix: string[] = [],
  ): ThemeSelector<T> {
    return Object.fromEntries(
      Object.entries(tokens).map(([name, value]) => {
        switch (typeof value) {
          case "string":
          case "number":
            return [name, `var(${intoVariableName([...prefix, name])})`];
          default:
            return [name, intoSelectors(value, [...prefix, name])];
        }
      }),
    ) as ThemeSelector<T>;
  }

  function intoVariables<T extends CSSProperties>(
    tokens: T,
    prefix: string[] = [],
  ): (readonly [string, string])[] {
    return Object.entries(tokens).flatMap(([name, value]) => {
      switch (typeof value) {
        case "string":
          return [[intoVariableName([...prefix, name]), value]] as const;
        case "number":
          return [[intoVariableName([...prefix, name]), `${value}px`]] as const;
        default:
          return intoVariables(value, [...prefix, name]);
      }
    });
  }

  function intoVariableName(path: string[]): string {
    return "--doctools-" + path.join("-");
  }
}

type CSSProperties = {
  [x: string]: string | number | CSSProperties;
};

type ThemeSelector<T> = T extends string | number
  ? string
  : T extends CSSProperties
    ? { [K in keyof T]: ThemeSelector<T[K]> }
    : never;

type PartialTheme<T> = T extends CSSProperties
  ? Partial<{ [K in keyof T]: PartialTheme<T[K]> }>
  : T;
