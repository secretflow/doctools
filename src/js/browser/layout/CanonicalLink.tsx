import type { Pep440Version } from "@renovatebot/pep440";
import { Helmet } from "react-helmet-async";

import {
  findStableVersion,
  usePathPatcher,
  useRepoContent,
  useSiteContent,
} from "../app";

export function CanonicalLink() {
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

  const link = (
    <Helmet>
      <link
        rel="canonical"
        href={window.location.origin + patched({ ref: stable.raw })}
      ></link>
    </Helmet>
  );

  if (current.label === "head") {
    return link;
  }

  if (
    (current.version &&
      stable.version &&
      ltBaseVersion(current.version, stable.version) < 0) ||
    current.label !== stable.label
  ) {
    return link;
  }

  return null;
}

const ltBaseVersion = (a: Pep440Version, b: Pep440Version) =>
  a.release[0] - b.release[0] ||
  a.release[1] - b.release[1] ||
  a.release[2] - b.release[2];
