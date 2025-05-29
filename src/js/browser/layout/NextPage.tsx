import { faChevronLeft, faChevronRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans, useLingui } from "@lingui/react/macro";
import { Typography } from "antd";
import { NavLink } from "react-router";
import { styled } from "styled-components";

import type { SidebarItem } from "../../docs/types";
import { getSidebar, iterSidebar, useFullContent, useSiteContent } from "../app";
import { breakpoint, theme } from "../theme";

import { LinkLoading } from "./LinkLoading";

export function NextPage() {
  const { t } = useLingui();

  const {
    path: { build: buildPath },
  } = useSiteContent();

  let {
    repo: { project },
    page: { suffix },
  } = useFullContent();

  const sidebar = getSidebar(project);
  const iter = iterSidebar(sidebar);

  let prev: SidebarItem | null = null;
  let curr: SidebarItem | null = null;
  let next: SidebarItem | null = null;

  suffix ||= ".";

  while (true) {
    const item = iter.next().value;

    if (item && item?.kind !== "doc") {
      continue;
    }

    prev = curr;
    curr = next;
    next = item ?? null;

    if (curr?.key === suffix) {
      break;
    }

    if (next === null) {
      break;
    }
  }

  if (suffix === ".") {
    if (next === null) {
      // next page fallback
      prev = null;
      curr = null;
      for (next of iterSidebar(sidebar)) {
        if (next.kind === "doc") {
          break;
        }
      }
    }
  } else {
    if (curr?.key !== suffix) {
      // invalid
      prev = null;
      next = null;
    } else if (prev === null && "." in project.module.sitemap) {
      // prev page fallback
      prev = {
        key: "",
        title: t`Homepage`,
        kind: "doc",
      };
    }
  }

  return (
    <PageLinkContainer>
      {prev ? (
        <PageLink to={buildPath({ ...project, suffix: prev.key })}>
          <Typography.Text style={{ fontWeight: 500 }}>
            <Trans>Previous page</Trans>
          </Typography.Text>
          <Typography.Text strong style={{ fontSize: "1rem" }}>
            <FontAwesomeIcon icon={faChevronLeft} />
            <span style={{ marginInlineStart: 6 }}>{prev.title}</span>
            <LinkLoading
              repo={project.repo}
              ref_={project.ref}
              lang={project.lang}
              suffix={prev.key}
              search={undefined}
              hash={undefined}
              style={{
                color: "inherit",
                marginInlineStart: 6,
                position: "relative",
                top: 2,
              }}
            />
          </Typography.Text>
        </PageLink>
      ) : (
        <div />
      )}
      {next ? (
        <PageLinkNext to={buildPath({ ...project, suffix: next.key })}>
          <Typography.Text style={{ fontWeight: 500 }}>
            <Trans>Next page</Trans>
          </Typography.Text>
          <Typography.Text strong style={{ fontSize: "1rem" }}>
            <LinkLoading
              repo={project.repo}
              ref_={project.ref}
              lang={project.lang}
              suffix={next.key}
              search={undefined}
              hash={undefined}
              style={{
                color: "inherit",
                marginInlineEnd: 6,
                position: "relative",
                top: 2,
              }}
            />
            <span style={{ marginInlineEnd: 6 }}>{next.title}</span>
            <FontAwesomeIcon icon={faChevronRight} />
          </Typography.Text>
        </PageLinkNext>
      ) : (
        <div />
      )}
    </PageLinkContainer>
  );
}

const PageLink = styled(NavLink)`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.5ch;
  justify-content: center;
  padding: 0.6rem 1rem;
  text-decoration: none;
  border: 1px solid rgb(239 239 239);
  border-radius: 6px;

  &:hover {
    border-color: ${theme.colors.fg.link};

    * {
      color: ${theme.colors.fg.link};
    }
  }

  ${breakpoint("mobileWidth")} {
    order: 1;
  }
`;

const PageLinkNext = styled(PageLink)`
  align-items: flex-end;
  text-align: end;

  ${breakpoint("mobileWidth")} {
    order: 0;
  }
`;

const PageLinkContainer = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-top: 2rem;

  ${breakpoint("mobileWidth")} {
    grid-template-columns: 1fr;
  }
`;
