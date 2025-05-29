import type { CSSProperties } from "react";
import { createPath } from "react-router";
import { styled } from "styled-components";

import { usePageNavigation, useSiteContent } from "../app";
import { theme } from "../theme";

import { Spinner } from "./Spinner";

export function LinkLoading({
  repo,
  ref_,
  lang,
  suffix,
  search,
  hash,
  style,
}: {
  repo: string;
  ref_: string | undefined;
  lang: string | undefined;
  suffix: string | undefined;
} & {
  search: string | undefined;
  hash: string | undefined;
  style?: CSSProperties;
}) {
  const {
    path: { build: buildPath },
  } = useSiteContent();
  const next = usePageNavigation();
  if (!next || !Object.keys(next.pathname).length) {
    return null;
  }
  const link = { repo, ref: ref_, lang, suffix };
  const p1 = createPath({
    ...next,
    pathname: buildPath({ ...link, ...next.pathname }),
  });
  const p2 = createPath({
    pathname: buildPath({ ...link }),
    search,
    hash,
  });
  if (decodeURIComponent(p1) === decodeURIComponent(p2)) {
    return <Indicator style={style} />;
  } else {
    return null;
  }
}

export const Indicator = styled(Spinner)`
  width: 1em;
  height: 1em;
  margin-inline: 4px 2px;
  color: ${theme.colors.fg.link};
`;

LinkLoading.Indicator = Indicator;
