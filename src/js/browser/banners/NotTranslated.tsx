import { faLanguage } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans, useLingui } from "@lingui/react/macro";
import { Alert } from "antd";

import { useRepoContent, useRepoExtras } from "../app";
import { quoteUrl } from "../error";
import { printLocaleNameIn } from "../i18n";

import { minimizedLocale } from "./SuggestLanguage";

export function NotTranslated() {
  const { i18n } = useLingui();

  const {
    repo: {
      project,
      message: { redirectedFrom: { matched: { lang } = {} } = {} },
    },
  } = useRepoContent();

  const { projectName } = useRepoExtras(project);

  if (!lang) {
    return null;
  }

  if (minimizedLocale(lang) !== minimizedLocale(project.lang)) {
    const versionName = `${projectName} ${project.ref}`;
    return (
      <Alert
        closable
        banner
        type="info"
        icon={<FontAwesomeIcon icon={faLanguage} width="14px" />}
        message={
          <Trans>
            Documentation for {quoteUrl(versionName)} has not yet been translated to{" "}
            {printLocaleNameIn(i18n.locale, lang)}. You are viewing the{" "}
            {printLocaleNameIn(i18n.locale, project.lang)} version instead.
          </Trans>
        }
      />
    );
  }

  return null;
}
