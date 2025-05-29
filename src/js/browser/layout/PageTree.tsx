import { faChevronRight, faExternalLink } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button, ConfigProvider, Flex, Menu, Typography } from "antd";
import type { ItemType, MenuItemType } from "antd/es/menu/interface";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import { Link, useLocation } from "react-router";
import { styled } from "styled-components";

import type { Sidebar, SidebarItem } from "../../docs/types";
import {
  getBreadcrumbs,
  getSidebar,
  usePartialPageContent,
  useRepoContent,
  useRepoExtras,
  useSiteContent,
} from "../app";

import { LinkLoading } from "./LinkLoading";

export function PageTree() {
  const {
    path: { build: buildPath },
  } = useSiteContent();

  const {
    repo: { project },
  } = useRepoContent();

  const { page: { suffix } = {} } = usePartialPageContent();

  const sidebar = getSidebar(project);

  const selected = getBreadcrumbs(sidebar, suffix)?.path || [];

  const [expanded, setExpanded] = useState<string[]>(selected);

  function expandToKey(key: string): SidebarItem | null {
    const next = getBreadcrumbs(sidebar, key);
    setExpanded((prev) => {
      if (next) {
        return next.path;
      } else {
        return prev;
      }
    });
    return next?.items.slice(-1).pop() ?? null;
  }

  function collapseFromKey(key: string) {
    setExpanded((prev) => {
      const idx = prev.indexOf(key);
      if (idx !== -1) {
        return prev.slice(0, idx);
      } else {
        return [];
      }
    });
  }

  const { chapterName } = useRepoExtras(project);

  function sidebarDataToMenu(sidebar: Sidebar): MenuItemType[] {
    return sidebar.map((item) => {
      const { key, title, kind, children } = item;

      let label: ReactNode = title;

      if (chapterName) {
        label = chapterName(item);
      }

      label = (
        <Flex align="center" gap={10} style={{ minWidth: 0, marginInlineEnd: 6 }}>
          <Typography.Text
            ellipsis
            style={{ fontSize: "inherit", color: "inherit", flex: "1 1 auto" }}
          >
            {label}
          </Typography.Text>
          {kind === "link" ? (
            <span>
              <FontAwesomeIcon
                icon={faExternalLink}
                fontSize={10}
                style={{ margin: "0 4px 0 8px" }}
              />
            </span>
          ) : null}
          <LinkLoading
            repo={project.repo}
            ref_={project.ref}
            lang={project.lang}
            suffix={key}
            search={undefined}
            hash={undefined}
            style={{ marginInline: 0 }}
          />
        </Flex>
      );

      switch (kind) {
        case "doc":
          label = <Link to={buildPath({ ...project, suffix: key })}>{label}</Link>;
          break;
        case "link":
          label = (
            <Link to={key} target="_blank">
              {label}
            </Link>
          );
          break;
        case "category":
          break;
      }

      return {
        label,
        key,
        title,
        children: children?.length ? sidebarDataToMenu(children) : undefined,
      } satisfies ItemType<MenuItemType>;
    });
  }

  const { pathname } = useLocation();

  const home = buildPath({ ...project });

  useEffect(() => {
    if (pathname.startsWith(home)) {
      const next = getBreadcrumbs(sidebar, pathname.slice(home.length));
      if (next) {
        setExpanded(next.path);
      }
    }
  }, [home, pathname, sidebar]);

  return (
    <ConfigProvider
      theme={{
        components: {
          Menu: {
            itemMarginBlock: 0,
            itemMarginInline: 0,
          },
        },
      }}
    >
      <SitemapMenu
        mode="inline"
        inlineIndent={12}
        items={sidebarDataToMenu(sidebar)}
        openKeys={expanded}
        selectedKeys={selected}
        onOpenChange={(willOpen) => {
          for (const key of willOpen) {
            if (!expanded.includes(key)) {
              expandToKey(key);
            }
          }
          for (const key of expanded) {
            if (!willOpen.includes(key)) {
              collapseFromKey(key);
            }
          }
        }}
        expandIcon={({
          isOpen,
          // @ts-expect-error using private API
          eventKey,
        }) => {
          return (
            <Button
              type="text"
              style={{
                padding: 0,
                height: "100%",
                aspectRatio: "1",
                width: "auto",
                margin: 0,
              }}
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (eventKey === undefined) {
                  return;
                }
                if (!isOpen) {
                  expandToKey(eventKey);
                } else {
                  collapseFromKey(eventKey);
                }
              }}
              icon={
                <FontAwesomeIcon
                  icon={faChevronRight}
                  className="expand-icon"
                  style={{ transform: isOpen ? "rotate(90deg)" : "none" }}
                />
              }
            />
          );
        }}
      />
    </ConfigProvider>
  );
}

const SitemapMenu = styled(Menu)`
  border-inline-end: none !important;

  a {
    color: inherit;
  }

  .ant-menu-title-content {
    font-size: 16px;
    font-weight: 500;
  }

  .ant-menu-submenu-title {
    padding-inline-end: 0 !important;
  }

  .ant-menu-submenu-selected:not(
      :has(.ant-menu-item-selected, .ant-menu-submenu-selected)
    )
    > .ant-menu-submenu-title {
    background-color: #e6f4ff;
  }

  /* .ant-menu-item, */
  .ant-menu-sub,
  .ant-menu-light {
    background-color: transparent !important;
  }
`;
