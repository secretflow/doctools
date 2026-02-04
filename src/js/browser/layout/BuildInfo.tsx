import { faEdit } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans, useLingui } from "@lingui/react/macro";
import { Popover, Typography } from "antd";
import { format, formatRelative } from "date-fns";
import { enUS, zhCN } from "date-fns/locale";
import type { ReactNode } from "react";
import { Fragment } from "react";

import { usePageContent } from "../app";

export function BuildInfo() {
  const {
    page: {
      content: { frontmatter },
    },
  } = usePageContent();

  const { i18n } = useLingui();

  const formatOptions = i18n.locale.startsWith("zh")
    ? { locale: zhCN }
    : { locale: enUS };

  const {
    git_last_modified_commit,
    git_last_modified_time,
    git_revision_commit,
    git_revision_time,
    git_owner,
    git_repo,
    git_origin_url,
  } = frontmatter ?? {};

  const rows: ReactNode[] = [];

  let intoLink: (inner: ReactNode, commit?: string) => ReactNode = (inner, commit) =>
    commit ? (
      <span>
        {inner} (<code>{commit.slice(0, 7)}</code>)
      </span>
    ) : (
      <span>{inner}</span>
    );

  if (git_origin_url?.includes("github.com") && git_owner && git_repo) {
    const intoLinkInner = intoLink;
    intoLink = (inner, commit) =>
      commit ? (
        <a
          href={`https://github.com/${git_owner}/${git_repo}/commit/${commit}`}
          target="_blank"
          rel="noreferrer nofollow"
        >
          {intoLinkInner(inner, commit)}
        </a>
      ) : (
        intoLinkInner(inner, commit)
      );
  }

  let displayTime: string | undefined;

  if (git_last_modified_time) {
    const modifiedDateTime = new Date(git_last_modified_time);
    rows.push(
      <Fragment key="git_last_modified_time">
        <Typography.Text>
          <Trans>Last modified</Trans>
        </Typography.Text>
        {intoLink(
          format(modifiedDateTime, "PPpp", formatOptions),
          git_last_modified_commit,
        )}
      </Fragment>,
    );
    displayTime = formatRelative(modifiedDateTime, new Date(), formatOptions);
  }

  if (git_revision_time) {
    const revisionDateTime = new Date(git_revision_time);
    rows.push(
      <Fragment key="git_revision_time">
        <Typography.Text>
          <Trans>Page built</Trans>
        </Typography.Text>
        {intoLink(format(revisionDateTime, "PPpp", formatOptions), git_revision_commit)}
      </Fragment>,
    );
    displayTime =
      displayTime || formatRelative(revisionDateTime, new Date(), formatOptions);
  }

  if (!displayTime) {
    return null;
  }

  return (
    <Popover
      content={
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "auto auto",
            gap: "0.1rem 0.5rem",
            lineHeight: 1,
            alignItems: "baseline",
          }}
        >
          {rows}
        </div>
      }
      mouseEnterDelay={0.8}
      placement="left"
    >
      <Typography.Text type="secondary" style={{ cursor: "help" }}>
        <FontAwesomeIcon
          icon={faEdit}
          style={{ display: "inline-block", width: "16px" }}
        />{" "}
        {displayTime}
      </Typography.Text>
    </Popover>
  );
}
