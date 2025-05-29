import type { CSSProperties } from "react";

import { usePageNavigation } from "../app";

import { Indicator } from "./LinkLoading";

export function RepoLoading({
  repo,
  ref_,
  lang,
  style,
}: {
  repo?: string | true;
  ref_?: string | true;
  lang?: string | true;
  style?: CSSProperties;
}) {
  const { pathname = {} } = usePageNavigation() ?? {};
  if (pathname.repo && (repo === true || repo === pathname.repo)) {
    return <Indicator style={style} />;
  }
  if (pathname.lang && (lang === true || lang === pathname.lang)) {
    return <Indicator style={style} />;
  }
  if (pathname.ref && (ref_ === true || ref_ === pathname.ref)) {
    return <Indicator style={style} />;
  }
  return null;
}
