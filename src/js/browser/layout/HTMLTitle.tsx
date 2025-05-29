import { useLingui } from "@lingui/react/macro";
import { useMemo } from "react";
import { Helmet } from "react-helmet-async";

import {
  getBreadcrumbs,
  getSidebar,
  usePartialPageContent,
  useRepoContent,
  useRepoExtras,
  useSiteContent,
} from "../app";

export function HTMLTitle() {
  const {
    repo: { project },
  } = useRepoContent();

  const { page: { suffix } = {} } = usePartialPageContent();

  const title1 = useMemo(
    () => getBreadcrumbs(getSidebar(project), suffix)?.items?.pop()?.title,
    [project, suffix],
  );

  const title2 = `${useRepoExtras(project).projectName} ${project.ref}`;

  const title = useHTMLTitle(title1, title2);

  return (
    <Helmet>
      <title>{title}</title>
    </Helmet>
  );
}

export function useHTMLTitle(...items: (string | number | undefined)[]) {
  return [...items, useTopLevelTitle()].filter(Boolean).join(" | ");
}

function useTopLevelTitle() {
  const {
    extras: { useSiteTitle },
  } = useSiteContent();
  const { t } = useLingui();
  return useSiteTitle?.() || t`SecretFlow Docs`;
}
