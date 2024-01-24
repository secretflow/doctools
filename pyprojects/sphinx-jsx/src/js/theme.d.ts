import type { I18n } from "@lingui/core";
import type { Trans } from "@lingui/react";
import type { Fragment, PropsWithChildren } from "react";

type BuiltinProps = {
  id: string | null;
  ids: string | null;
  className: string | null;
};

type VoidElementProps = BuiltinProps;

type ElementProps = PropsWithChildren<BuiltinProps>;

export declare const _url: (type: string, base: string | null, ref: string) => string;
export declare const _jsx: unknown;
export declare const _jsxs: unknown;
export declare const _Fragment: typeof Fragment;
export declare const _Trans: typeof Trans;
export declare const _gettext: I18n["_"];
