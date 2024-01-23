import type { I18n } from "@lingui/core";
import type { Trans } from "@lingui/react";
import type { Fragment, FC, PropsWithChildren } from "react";

type IntrinsicProps = {
  id?: string;
  className?: string;
};

export declare const _url: (type: string, base: string | null, ref: string) => string;
export declare const _jsx: unknown;
export declare const _jsxs: unknown;
export declare const _Fragment: typeof Fragment;
export declare const _Trans: typeof Trans;
export declare const _gettext: I18n["_"];

export declare const Title: FC<PropsWithChildren<IntrinsicProps>>;

export declare const Link: FC<
  PropsWithChildren<{ href: ReturnType<typeof _url> } & IntrinsicProps>
>;

export declare const Image: FC<
  PropsWithChildren<{ src: ReturnType<typeof _url> } & IntrinsicProps>
>;
