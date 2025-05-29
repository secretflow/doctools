import { faLanguage } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Alert } from "antd";
import { useState } from "react";
import { Link } from "react-router";

import { useProjectLanguages, usePathPatcher, useRepoContent } from "../app";
import { printLocaleNameIn } from "../i18n";
import { RepoLoading } from "../layout/RepoLoading";

export function SuggestLanguage() {
  const patched = usePathPatcher();

  const {
    repo: { project },
  } = useRepoContent();

  const available = useProjectLanguages({ ...project });

  const [closed, setClosed] = useState(false);

  if (closed) {
    return null;
  }

  const suggested = (() => {
    const current = minimizedLocale(project.lang);
    for (const acceptLang of globalThis.navigator.languages) {
      const accepted = minimizedLocale(acceptLang);
      if (accepted === current) {
        break;
      }
      const suggested = available.findIndex((t) => minimizedLocale(t) === accepted);
      if (suggested !== -1) {
        return available[suggested];
      }
    }
    return null;
  })();

  if (suggested === null) {
    return null;
  }

  const localeName = printLocaleNameIn(suggested);

  return (
    <Alert
      banner
      closable
      type="success"
      icon={<FontAwesomeIcon icon={faLanguage} width="14px" />}
      message={
        <Trans>
          This page is also available in{" "}
          <Link
            to={patched({ lang: suggested })}
            style={{
              padding: 0,
              height: "initial",
              lineHeight: "1em",
              display: "inline-flex",
              alignItems: "center",
            }}
          >
            <span>{localeName}</span>
            <RepoLoading lang={suggested} />
          </Link>
          .
        </Trans>
      }
      onClose={() => setClosed(true)}
    />
  );
}

export function minimizedLocale(tag: string): string {
  try {
    return new Intl.Locale(tag).minimize().baseName;
  } catch {
    return tag;
  }
}
