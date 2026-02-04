import { faMessage } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans, useLingui } from "@lingui/react/macro";
import { Typography } from "antd";
import newGitHubIssueURL from "new-github-issue-url";

import { useFullContent } from "../app";

export function SuggestEdit() {
  const { t } = useLingui();

  const {
    page: { content },
    repo: { project },
  } = useFullContent();

  const githubUrl = content?.frontmatter?.git_origin_url;

  if (!githubUrl?.includes("github.com")) {
    return null;
  }

  const body = [
    t`### Suggest an edit to documentation`,
    t`<!-- Please describe the issue you encountered -->`,
    t`<!-- Do not edit below this line -->`,
    `<details>
      <summary>${t`Technical info`}</summary>
      <ul>
        <li>${t`Page`} <a href="${window.location.href}">${
          window.location.href
        }</a></li>
        <li>${t`Source`} <a href="${githubUrl}">${githubUrl}</a></li>
        <li>${t`User-Agent`} <code>${navigator.userAgent}</code></li>
      </ul>
    </details>`,
  ].join("\n\n");

  const newIssueURL = newGitHubIssueURL({
    user: "secretflow",
    repo: project.repo,
    body,
  });

  return (
    <Typography.Link
      type="secondary"
      href={newIssueURL}
      rel="noopener noreferrer nofollow"
      target="_blank"
    >
      <FontAwesomeIcon
        icon={faMessage}
        style={{ display: "inline-block", width: "16px" }}
      />{" "}
      <Trans>Suggest an edit</Trans>
    </Typography.Link>
  );
}
