import { createGlobalStyle } from 'styled-components';

type CSSVariables = { [x: string]: string | CSSVariables };

type ThemeTokens = {
  openapi: {
    backgroundColors: {
      default: string;
      info: string;
      warning: string;
    };
    colors: {
      default: string;
      inverted: string;
      muted: string;
      link: string;
      blue: string;
      green: string;
      red: string;
      yellow: string;
      magenta: string;
      cyan: string;
      border: string;
      neutral: string;
    };
    typography: {
      sans: string;
      monospace: string;
    };
    spacing: {
      xs: string;
      s: string;
      sm: string;
      m: string;
      ml: string;
    };
  };
};

function createTheme(tokens: ThemeTokens): {
  ThemeVariables: ReturnType<typeof createGlobalStyle>;
  vars: ThemeTokens;
  tokens: ThemeTokens;
} {
  const reduceVars = (root: CSSVariables): Record<string, string> =>
    Object.fromEntries(
      Object.entries(root).flatMap(([key, value]) =>
        typeof value === 'string'
          ? [[key, value]]
          : Object.entries(reduceVars(value)).map(([k, v]) => [`${key}-${k}`, v]),
      ),
    );

  const mapVars = <T extends CSSVariables>(root: T, prefix = ''): T =>
    Object.fromEntries(
      Object.entries(root).map(([key, value]) => [
        key,
        typeof value === 'string'
          ? `var(--${prefix}${key})`
          : mapVars(value, `${prefix}${key}-`),
      ]),
    ) as T;

  const mapping = reduceVars(tokens);

  const ThemeVariables = createGlobalStyle`
    :root {
      ${Object.entries(mapping)
        .map(([key, value]) => `--${key}: ${value};`)
        .join('')}
    }
  `;
  const vars = mapVars(tokens);

  return { ThemeVariables, vars, tokens };
}

export const lightTheme = createTheme({
  openapi: {
    backgroundColors: {
      default: '#fdfdfe',
      info: '#eef9fd',
      warning: '#fff8e6',
    },
    colors: {
      default: 'rgb(0 0 0 / 88%)',
      inverted: '#fdfdfe',
      muted: '#4f5a66',
      link: 'rgb(0, 96, 230)',
      blue: '#61afef',
      red: '#e06c75',
      green: '#98c379',
      yellow: '#e5c07b',
      magenta: '#c678dd',
      cyan: '#56b6c2',
      neutral: '#abb2bf',
      border: '#eaeaea',
    },
    typography: {
      sans: "Inter, Noto Sans SC, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif",
      monospace: "'Roboto Mono', monospace",
    },
    spacing: {
      xs: '5px',
      s: '10px',
      sm: '15px',
      m: '20px',
      ml: '30px',
    },
  },
});
