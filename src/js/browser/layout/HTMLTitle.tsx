import { useLingui } from "@lingui/react/macro";
import { Helmet } from "react-helmet-async";

import {
  getBreadcrumbs,
  useRepoContent,
  useSiteContent,
  usePageContent,
  getSidebar,
  useRepoExtras,
  usePartialPageContent,
} from "../app";

export function HTMLTitle() {
  const {
    repo: { project },
  } = useRepoContent();

  // const { page: { suffix } = {} } = usePartialPageContent();

  // const title1 = useMemo(
  //   () => getBreadcrumbs(getSidebar(project), suffix)?.items?.pop()?.title,
  //   [project, suffix],
  // );
  // console.log(getBreadcrumbs(getSidebar(project), suffix),'getBreadcrumbs(getSidebar(project), suffix)')
  //  console.log(title1)
  const title2 = `${useRepoExtras(project).projectName} ${project.ref}`;

  // const title = useHTMLTitle(title1, title2);
  const title = useTopLevelTitle() || title2;
  return (
    <Helmet>
      <title>{title}</title>
    </Helmet>
  );
}

export function useHTMLTitle(...items: (string | number | undefined)[]) {
  return [...items, useTopLevelTitle()].filter(Boolean).join("-");
}

function useTopLevelTitle() {
  const {
    extras: { useSiteTitle },
  } = useSiteContent();
  const { page } = usePageContent();
  const { repo } = useRepoContent();
  const { page: { suffix } = {} } = usePartialPageContent();
  const { t } = useLingui();
  const breadcrumbs = getBreadcrumbs(getSidebar(repo?.project), suffix);
  return useSiteTitle?.(page, repo, breadcrumbs) || t`SecretFlow Docs`;
}
