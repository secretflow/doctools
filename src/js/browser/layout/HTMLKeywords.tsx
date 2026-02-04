import { useLingui } from "@lingui/react/macro";
import { Helmet } from "react-helmet-async";

import { useRepoContent, useRepoExtras, useSiteContent, usePageContent } from "../app";

export function HTMLKeywords() {
  const {
    repo: { project },
  } = useRepoContent();

  const defaultKeyword = `${useRepoExtras(project).projectName} ${project.ref}`;

  const keywords = useKeywordsTitle() || defaultKeyword;
  return (
    <Helmet>
      <meta name="keywords" content={keywords} />
    </Helmet>
  );
}

function useKeywordsTitle() {
  const {
    extras: { useSiteKeywords },
  } = useSiteContent();
  const { page } = usePageContent();
  const { repo } = useRepoContent();
  const { t } = useLingui();
  return useSiteKeywords?.(page, repo) || t`SecretFlow Docs`;
}
