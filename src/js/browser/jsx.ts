import { Fragment } from "react";
import { jsxDEV } from "react/jsx-dev-runtime";
import { jsx, jsxs } from "react/jsx-runtime";

export const runtime = import.meta.env.DEV
  ? {
      Fragment,
      jsx: jsxDEV as typeof jsx,
      jsxs: jsxDEV as typeof jsxs,
      jsxDEV,
      development: true,
    }
  : {
      Fragment,
      jsx,
      jsxs,
    };
