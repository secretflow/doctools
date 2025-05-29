import { css } from "styled-components";

export const ansi = css`
  --ansi-black: hsl(214.98deg 31% 16.95%);
  --ansi-red: hsl(349.68deg 74.8% 44.32%);
  --ansi-green: hsl(149.97deg 73.96% 35.59%);
  --ansi-yellow: hsl(45.42deg 72.54% 47.34%);
  --ansi-blue: hsl(221.33deg 69.57% 49.3%);
  --ansi-cyan: hsl(189.96deg 76.91% 39.54%);
  --ansi-magenta: hsl(294.63deg 68.77% 49.91%);

  .ansi-black-fg,
  .ansi30 {
    color: var(--ansi-black) !important;
  }

  .ansi-red-fg,
  .ansi31 {
    color: var(--ansi-red) !important;
  }

  .ansi-green-fg,
  .ansi32 {
    color: var(--ansi-green) !important;
  }

  .ansi-yellow-fg,
  .ansi33 {
    color: var(--ansi-yellow) !important;
  }

  .ansi-blue-fg,
  .ansi34 {
    color: var(--ansi-blue) !important;
  }

  .ansi-magenta-fg,
  .ansi35 {
    color: var(--ansi-magenta) !important;
  }

  .ansi-cyan-fg,
  .ansi36 {
    color: var(--ansi-cyan) !important;
  }

  .ansi-white-intense-fg,
  .ansi97 {
    color: #000 !important;
  }

  .ansi-black-bg,
  .ansi40 {
    color: #fafafa !important;
    background-color: var(--ansi-black) !important;
  }

  .ansi-red-bg,
  .ansi41 {
    color: #fafafa !important;
    background-color: var(--ansi-red) !important;
  }

  .ansi-green-bg,
  .ansi42 {
    color: #fafafa !important;
    background-color: var(--ansi-green) !important;
  }

  .ansi-yellow-bg,
  .ansi43 {
    color: #fafafa !important;
    background-color: var(--ansi-yellow) !important;
  }

  .ansi-blue-bg,
  .ansi44 {
    color: #fafafa !important;
    background-color: var(--ansi-blue) !important;
  }

  .ansi-magenta-bg,
  .ansi45 {
    color: #fafafa !important;
    background-color: var(--ansi-magenta) !important;
  }

  .ansi-cyan-bg,
  .ansi46 {
    color: #fafafa !important;
    background-color: var(--ansi-cyan) !important;
  }

  .ansi-bold,
  .ansi1 {
    font-weight: 600 !important;
  }

  .ansi-underline,
  .ansi4 {
    text-decoration: underline !important;
  }

  .ansi-concealed,
  .ansi8 {
    visibility: visible !important;
    color: transparent !important;
  }

  .ansi9 {
    text-decoration: line-through !important;
  }
`;
