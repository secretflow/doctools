import { faLinkSlash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Alert } from "antd";
import { useLocation } from "react-router";

import { useRepoContent, useSiteContent } from "../app";
import { quoteUrl } from "../error";
import { wordBreak } from "../page/whitespace";

export function RedirectedFrom() {
  const location = useLocation();

  const {
    path: { parse: parsePath },
  } = useSiteContent();

  const {
    repo: {
      project,
      message: { redirectedFrom },
    },
  } = useRepoContent();

  if (!redirectedFrom) {
    return null;
  }

  const { matched, printed } = redirectedFrom;

  const { repo, ref, lang } = matched;

  let { suffix: oldPath } = matched;
  let { suffix: newPath } = parsePath(location.pathname) ?? {};

  oldPath = oldPath?.replace(/\/+$/, "");
  newPath = newPath?.replace(/\/+$/, "");

  if (
    repo !== project.repo ||
    (ref && ref !== project.ref) ||
    (lang && lang !== project.lang) ||
    (oldPath && oldPath !== newPath)
  ) {
    return (
      <Alert
        closable
        banner
        type="info"
        icon={<FontAwesomeIcon icon={faLinkSlash} width="14px" />}
        message={<Trans>Redirected from {wordBreak(quoteUrl(printed))}</Trans>}
      />
    );
  }

  return null;
}
