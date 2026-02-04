import { useEffect, useState } from "react";
import { Helmet } from "react-helmet-async";

import { useRepoContent, useRepoExtras, useSiteContent, usePageContent } from "../app";

export function HTMLDescription() {
  const {
    repo: { project },
  } = useRepoContent();

  const {
    extras: { getSiteDescription },
  } = useSiteContent();
  const { page } = usePageContent();
  const { repo } = useRepoContent();
  const defaultDescription = `${useRepoExtras(project).projectName} ${project.ref}`;

  // const description = useDescriptionTitle() || defaultDescription

  const [description, setDescription] = useState("");

  useEffect(() => {
    async function loadDesc() {
      const desc = (await getSiteDescription?.(page, repo)) || defaultDescription;
      setDescription(desc);
    }
    loadDesc();
  }, [page, repo, defaultDescription, getSiteDescription]);

  return (
    <Helmet>
      <meta name="description" content={description} />
    </Helmet>
  );
}
