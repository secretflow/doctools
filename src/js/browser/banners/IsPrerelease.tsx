import { faCodeBranch } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import type { Pep440Version } from "@renovatebot/pep440";
import { Alert, Typography } from "antd";
import { Link } from "react-router";

import {
  findStableVersion,
  usePathPatcher,
  useRepoContent,
  useSiteContent,
} from "../app";
import { RepoLoading } from "../layout/RepoLoading";

export function IsPrerelease() {
  const patched = usePathPatcher();

  const { versions } = useSiteContent();
  const {
    repo: { project },
  } = useRepoContent();

  const { head, tags, rest } = versions[project.repo];
  const sorted = [...tags];
  sorted.reverse();

  const [, stable] = findStableVersion({ sorted }) ?? [];

  const current = [...tags, ...head, ...rest].find((r) => r.raw === project.ref);

  if (!current?.label || !stable?.label || current.raw === stable.raw) {
    return null;
  }

  const button = (
    <Link
      to={patched({ ref: stable.raw })}
      style={{
        padding: 0,
        height: "initial",
        lineHeight: "1em",
        display: "inline-flex",
        alignItems: "center",
      }}
    >
      <span>{stable.raw}</span>
      <RepoLoading ref_={stable.raw} />
    </Link>
  );

  if (current.label === "head") {
    return (
      <Alert
        banner
        type="warning"
        icon={<FontAwesomeIcon icon={faCodeBranch} width="14px" />}
        message={
          <Trans>
            This is documentation for an{" "}
            <Typography.Text strong>unreleased</Typography.Text> version. {button} is
            the <Typography.Text strong>latest {stable.label}</Typography.Text> version.
          </Trans>
        }
      />
    );
  }

  if (
    (current.version &&
      stable.version &&
      ltBaseVersion(current.version, stable.version) < 0) ||
    current.label !== stable.label
  ) {
    if (current.label && current.label !== stable.label) {
      return (
        <Alert
          banner
          type="info"
          icon={<FontAwesomeIcon icon={faCodeBranch} width="14px" />}
          message={
            <Trans>
              This is documentation for <strong>{current.label}</strong> version{" "}
              {current.raw}; {button} is the <strong>latest {stable.label}</strong>{" "}
              version.
            </Trans>
          }
          closable
        />
      );
    } else {
      return (
        <Alert
          banner
          type="info"
          icon={<FontAwesomeIcon icon={faCodeBranch} width="14px" />}
          message={
            <Trans>
              This is documentation for version {current.raw}; {button} is the{" "}
              <strong>latest</strong> {stable.label} version.
            </Trans>
          }
          closable
        />
      );
    }
  }

  return null;
}

const ltBaseVersion = (a: Pep440Version, b: Pep440Version) =>
  a.release[0] - b.release[0] ||
  a.release[1] - b.release[1] ||
  a.release[2] - b.release[2];
