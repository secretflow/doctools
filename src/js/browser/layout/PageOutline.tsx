import { Anchor } from "antd";
import type { AnchorLinkItemProps } from "antd/es/anchor/Anchor";
import { Fragment, useEffect, useMemo, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router";
import { styled } from "styled-components";

import type { OutlineItem } from "../../docs/types";
import { usePageContent } from "../app";
import { useScrollParent } from "../page/anchoring";
import { useThemeToken } from "../theme";

export function PageOutline() {
  const { hash, key } = useLocation();

  const {
    page: {
      content: { outline },
    },
  } = usePageContent();

  const scrollable = useScrollParent();

  const { items, hrefs } = useMemo(
    () => getAnchorItems(outline ? [...outline] : []),
    [outline],
  );

  const [current, setCurrent] = useState<string>();

  const {
    dimensions: { scrollOffset },
  } = useThemeToken();

  useEffect(() => {
    const parent: HTMLElement | Document | null = scrollable();
    if (!parent) {
      return;
    }
    let target: EventTarget;
    if (parent === document.documentElement) {
      target = document;
    } else {
      target = parent;
    }
    const listener = () => {
      const positions = [...hrefs].flatMap((id) => {
        const anchor = document.getElementById(id);
        if (!anchor) {
          return [];
        } else {
          const { offsetTop } = anchor;
          return [{ id, offsetTop }];
        }
      });
      positions.sort(({ offsetTop: y1 }, { offsetTop: y2 }) => y1 - y2);
      const scrolled = Math.ceil(parent.scrollTop + scrollOffset);
      let scrolledId: string | undefined = undefined;
      for (const { offsetTop, id } of positions) {
        if (scrolled < offsetTop) {
          setCurrent(scrolledId);
          return;
        }
        scrolledId = id;
      }
      setCurrent(scrolledId);
    };
    target.addEventListener("scroll", listener, { passive: true });
    return () => target.removeEventListener("scroll", listener);
  }, [hrefs, scrollOffset, scrollable]);

  const notScrollable = useRef<HTMLDivElement>(null);

  const navigate = useNavigate();

  if (!outline) {
    return null;
  }

  return (
    <Fragment>
      <Anchor2
        key={key}
        affix={false}
        items={items}
        onClick={(evt, link) => {
          evt.preventDefault();
          if (evt.metaKey) {
            window.open(link.href, "_blank");
          } else {
            navigate(link.href);
          }
        }}
        getCurrentAnchor={() => (current ? `#${current}` : hash)}
        getContainer={() => notScrollable.current ?? document.body}
        style={{ maxHeight: "unset" }}
      />
      <div ref={notScrollable} style={{ display: "none" }} />
    </Fragment>
  );
}

function getAnchorItems(outline: OutlineItem[]): {
  hrefs: Set<string>;
  items: AnchorLinkItemProps[];
} {
  const hrefs: Set<string> = new Set();
  const items: AnchorLinkItemProps[] = [];
  let depth = 0;
  while (outline.length) {
    const item = outline.shift();
    if (!item) {
      break;
    }
    if (depth === 0) {
      depth = item.depth;
    }
    if (item.depth > depth) {
      outline.unshift(item);
      const { hrefs: inner, items: children } = getAnchorItems(outline);
      const lastItem = items.pop();
      if (!lastItem) {
        throw new Error("unreachable");
      }
      inner.forEach((k) => hrefs.add(k));
      items.push({ ...lastItem, children });
    } else if (item.depth < depth) {
      outline.unshift(item);
      break;
    } else {
      hrefs.add(item.id);
      items.push({
        key: item.id,
        href: `#${item.id}`,
        title: item.title,
      });
    }
  }
  return { hrefs, items };
}

const Anchor2 = styled(Anchor)`
  .ant-anchor-link-title {
    margin: 0;
    font-size: 0.8rem;
    line-height: 1.4 !important;
    overflow-wrap: break-word !important;
    white-space: normal !important;

    &:not(.ant-anchor-link-title-active) {
      color: rgb(88 93 100) !important;
    }
  }

  .ant-anchor-link {
    padding-block: 0 !important;
    padding-inline-start: 8px !important;
    margin-block: 8px;
  }

  > .ant-anchor > .ant-anchor-link {
    padding-inline-start: 12px !important;
  }
`;
