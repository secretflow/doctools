import { faGitAlt, faGithub, faGitlabSquare } from "@fortawesome/free-brands-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Typography } from "antd";

import { usePageContent } from "../app";

export function ViewPageSource() {
  const {
    page: { content },
  } = usePageContent();

  let githubUrl = content.frontmatter?.git_origin_url;

  if (githubUrl) {
    if (!githubUrl.includes("github.com")) {
      githubUrl = githubUrl.replace(/\/tree\//, "/blob/");
    }
    let icon = faGitAlt;
    if (githubUrl.includes("github")) {
      icon = faGithub;
    } else if (githubUrl.includes("gitlab")) {
      icon = faGitlabSquare;
    }
    return (
      <Typography.Link
        type="secondary"
        href={githubUrl}
        rel="noopener noreferrer nofollow"
        target="_blank"
      >
        <FontAwesomeIcon
          icon={icon}
          style={{ display: "inline-block", width: "16px" }}
        />{" "}
        <Trans>View page source</Trans>
      </Typography.Link>
    );
  } else {
    return null;
  }
}
