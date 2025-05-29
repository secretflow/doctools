import { faChevronRight, faEllipsis, faHome } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Dropdown, Typography } from "antd";
import type { ItemType } from "antd/es/menu/interface";
import type { ReactElement } from "react";
import { Fragment } from "react";
import { Link, useNavigate } from "react-router";
import { css, styled } from "styled-components";

import type { SidebarItem } from "../../docs/types";
import {
  getBreadcrumbs,
  getSidebar,
  useFullContent,
  usePageNavigation,
  useRepoExtras,
  useSiteContent,
} from "../app";
import { wordBreak } from "../page/whitespace";
import { theme } from "../theme";

import { LinkLoading } from "./LinkLoading";
import { Spinner } from "./Spinner";

export function Breadcrumbs() {
  const navigate = useNavigate();

  const navigation = usePageNavigation();

  const {
    path: { build: buildPath },
  } = useSiteContent();

  const {
    repo: { project },
    page: { suffix },
  } = useFullContent();

  const { projectName, chapterName } = useRepoExtras(project);

  const sidebar = getSidebar(project);

  const found = getBreadcrumbs(sidebar, suffix);

  if (!found?.items) {
    return null;
  }

  const children: ReactElement[] = [];

  children.push(
    <BreadcrumbLink key="$home" to={buildPath({ ...project })}>
      {navigation?.pathname.suffix === "" ? (
        <Spinner style={{ width: 14, height: 14 }} />
      ) : (
        <FontAwesomeIcon icon={faHome} style={{ width: 14, height: 14 }} />
      )}
      <Typography.Text style={{ fontSize: "inherit", marginInlineStart: 6 }} ellipsis>
        {projectName}
      </Typography.Text>
    </BreadcrumbLink>,
  );

  let visible = found.items;
  let abridged: SidebarItem[] = [];

  if (found.items.length > 6) {
    visible = [found.items[0], ...found.items.slice(-2)];
    abridged = found.items.slice(1, -2);
  }

  for (let i = 0; i < visible.length; i++) {
    const item = visible[i];

    children.push(
      <Typography.Text key={`${item.key}-separator`} type="secondary">
        <FontAwesomeIcon icon={faChevronRight} fontSize="0.7rem" />
      </Typography.Text>,
    );

    if (i === 1 && abridged.length) {
      children.push(
        <Fragment key={`${item.key}-abridged`}>
          <Dropdown
            menu={{
              items: abridged.map((item): ItemType => {
                const { key } = item;
                const label = chapterName(item);
                switch (item.kind) {
                  case "doc":
                    return {
                      key,
                      label: (
                        <Typography.Text>
                          <span>{wordBreak(label)}</span>
                          <LinkLoading
                            repo={project.repo}
                            ref_={project.ref}
                            lang={project.lang}
                            suffix={key}
                            search={undefined}
                            hash={undefined}
                            style={{
                              width: 12,
                              height: 12,
                              marginInlineStart: 3,
                              color: "inherit",
                            }}
                          />
                        </Typography.Text>
                      ),
                      onClick: (e) => {
                        const href = buildPath({ ...project, suffix: key });
                        if (e.domEvent.metaKey || e.domEvent.ctrlKey) {
                          window.open(href, "_blank");
                        } else {
                          navigate(href);
                        }
                      },
                    };
                  case "link":
                    return { key, label: wordBreak(label) };
                  case "category":
                    return {
                      key,
                      label: wordBreak(label),
                      type: "group",
                      children: [],
                    };
                }
              }),
            }}
          >
            <BreadcrumbText
              type="secondary"
              style={{ cursor: "pointer" }}
              role="button"
            >
              <FontAwesomeIcon icon={faEllipsis} fontSize="0.7rem" />
            </BreadcrumbText>
          </Dropdown>
          <Typography.Text type="secondary">
            <FontAwesomeIcon icon={faChevronRight} fontSize="0.7rem" />
          </Typography.Text>
        </Fragment>,
      );
    }

    switch (item.kind) {
      case "doc":
        children.push(
          <BreadcrumbLink
            key={item.key}
            to={buildPath({ ...project, suffix: item.key })}
          >
            <Typography.Text
              strong={item.key === suffix}
              style={{ fontSize: "inherit" }}
              ellipsis
            >
              {wordBreak(chapterName(item))}
            </Typography.Text>
            <LinkLoading
              repo={project.repo}
              ref_={project.ref}
              lang={project.lang}
              suffix={item.key}
              search={undefined}
              hash={undefined}
              style={{
                width: 12,
                height: 12,
                marginInlineStart: 3,
                color: "inherit",
              }}
            />
          </BreadcrumbLink>,
        );
        break;
      case "category":
        children.push(
          <BreadcrumbText key={item.key}>
            <Typography.Text style={{ fontSize: "inherit" }} ellipsis>
              {wordBreak(chapterName(item))}
            </Typography.Text>
          </BreadcrumbText>,
        );
        break;
      default:
        throw new Error("unreachable");
    }
  }

  return <BreadcrumbFrame>{children}</BreadcrumbFrame>;
}

const BreadcrumbFrame = styled.nav`
  position: relative;
  left: -6px;
  display: flex;
  flex-flow: row wrap;
  gap: 1ch;
  align-items: center;
  font-size: 0.9rem;
`;

const breadcrumbButton = css`
  padding: 2px 1ch;
  line-height: 1.4;
  border-radius: 8px;

  &:hover {
    background-color: rgb(0 0 0 / 4%);
  }
`;

const BreadcrumbLink = styled(Link)`
  ${breadcrumbButton}
  display: inline-flex;
  gap: 0.8ch;
  align-items: center;
  max-width: 100%;
  color: ${theme.colors.fg.default};
  text-decoration: none;
`;

const BreadcrumbText = styled(Typography.Text)`
  ${breadcrumbButton}
  max-width: 100%;
  user-select: none;
`;
