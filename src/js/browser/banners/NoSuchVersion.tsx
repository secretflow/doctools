import { faCodeBranch } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Alert } from "antd";

import { useRepoContent, useRepoExtras } from "../app";
import { quoteUrl } from "../error";

export function NoSuchVersion() {
  const {
    repo: {
      project,
      message: { redirectedFrom: { matched: { ref } = {} } = {} },
    },
  } = useRepoContent();

  const { projectName } = useRepoExtras(project);

  if (
    ref &&
    !["stable", "main", "master", "latest", "HEAD"].includes(ref) &&
    ref !== project.ref
  ) {
    return (
      <Alert
        closable
        banner
        type="warning"
        icon={<FontAwesomeIcon icon={faCodeBranch} width="14px" />}
        message={
          <Trans>
            {quoteUrl(projectName)} does not have documentation for version{" "}
            {quoteUrl(ref)}.
          </Trans>
        }
      />
    );
  }

  return null;
}
