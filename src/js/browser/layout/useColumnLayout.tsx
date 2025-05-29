import { faBook, faListUl, faRightToBracket } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Button } from "antd";
import type { ComponentProps, ReactNode } from "react";
import { Fragment, useCallback, useLayoutEffect, useRef, useState } from "react";
import type { ImperativePanelHandle, Panel, PanelGroup } from "react-resizable-panels";
import { PanelResizeHandle } from "react-resizable-panels";
import { useLocation } from "react-router";
import { styled } from "styled-components";

import { useClientMeasurements, usePageNavigation } from "../app";
import { theme, useThemeToken } from "../theme";

import { Spinner } from "./Spinner";

type PanelSide = "left" | "center" | "right";

export type ColumnLayout = {
  left: ComponentProps<typeof Panel>;
  center: ComponentProps<typeof Panel>;
  right: ComponentProps<typeof Panel>;
  outer: ComponentProps<typeof PanelGroup>;
  resizeHandle: ReactNode;
  toolbar: ReactNode;
};

export function useColumnLayout(): ColumnLayout {
  const { clientWidth, smallScreen } = useClientMeasurements();

  const {
    dimensions: { navbarHeight, mobileToolbarHeight },
  } = useThemeToken();

  // https://www.omnicalculator.com/math/slope-intercept-form
  // x1 = 1352, y1 = 22
  // x2 = 1920, y2 = 15
  const sidebarWidth = Math.min(Math.max(-0.01232394 * clientWidth + 38.662, 15), 22);

  const ref1 = useRef<ImperativePanelHandle>(null);
  const ref2 = useRef<ImperativePanelHandle>(null);
  const ref3 = useRef<ImperativePanelHandle>(null);

  const outer: ComponentProps<typeof PanelGroup> = {
    direction: "horizontal",
  };

  const left: ComponentProps<typeof Panel> = {
    collapsible: true,
    ref: ref1,
  };

  const center: ComponentProps<typeof Panel> = {
    collapsible: true,
    ref: ref2,
    style: {
      overflow: "visible",
      height: "unset",
      minWidth: 0,
    },
  };

  const right: ComponentProps<typeof Panel> = {
    collapsible: true,
    ref: ref3,
  };

  let resizeHandle: ReactNode;

  const [revealed, setRevealed] = useState<PanelSide>();

  const revealPanel = useCallback((side: PanelSide | undefined) => {
    setRevealed(side);
    switch (side) {
      case "left":
        ref1.current?.expand();
        ref1.current?.resize(100);
        ref2.current?.collapse();
        ref3.current?.collapse();
        break;
      case "center":
        ref1.current?.collapse();
        ref2.current?.expand();
        ref2.current?.resize(100);
        ref3.current?.collapse();
        break;
      case "right":
        ref1.current?.collapse();
        ref2.current?.collapse();
        ref3.current?.expand();
        ref3.current?.resize(100);
        break;
      case undefined: {
        ref1.current?.expand();
        ref2.current?.expand();
        ref3.current?.expand();
      }
    }
  }, []);

  if (!smallScreen) {
    left.defaultSize = sidebarWidth;
    left.minSize = 18;
    left.maxSize = 30;
    right.defaultSize = sidebarWidth;
    right.minSize = 18;
    right.maxSize = 25;
    center.defaultSize = 100 - 2 * sidebarWidth;

    resizeHandle = <ResizeHandle />;

    const sidebarHeight = `calc(100vh - ${navbarHeight}px)`;

    outer.style = {
      overflow: "visible",
      minHeight: sidebarHeight,
      flex: "1 1 auto",
    };
    left.style = {
      overflow: "auto",
      position: "sticky",
      height: sidebarHeight,
      top: navbarHeight,
    };
    right.style = {
      overflow: "auto",
      position: "sticky",
      height: sidebarHeight,
      top: navbarHeight,
    };
  } else {
    left.defaultSize = 0;
    right.defaultSize = 0;
    center.defaultSize = 100;

    resizeHandle = <ResizeHandle style={{ display: "none" }} />;

    const scrollOffset = navbarHeight + mobileToolbarHeight;
    const contentHeight = `calc(100vh - ${scrollOffset}px)`;

    outer.style = {
      overflow: "clip",
    };
    left.style = right.style = {
      overflow: "auto",
      position: "sticky",
      height: contentHeight,
      top: scrollOffset,
      zIndex: 1,
      background: "#ffffff",
    };
  }

  useLayoutEffect(() => {
    if (!smallScreen) {
      revealPanel(undefined);
    }
  }, [revealPanel, smallScreen]);

  const { pathname, hash } = useLocation();

  useLayoutEffect(() => {
    void pathname;
    void hash;
    if (smallScreen) {
      revealPanel("center");
    }
  }, [hash, pathname, revealPanel, smallScreen]);

  let toolbar: ReactNode = null;

  if (smallScreen) {
    toolbar = (
      <SidebarToolbarContainer>
        <SidebarToolbar revealed={revealed} revealPanel={revealPanel} />
      </SidebarToolbarContainer>
    );
  }

  return {
    left,
    center,
    right,
    outer,
    resizeHandle,
    toolbar,
  };
}

function SidebarToolbar({
  revealed,
  revealPanel,
}: {
  revealed: PanelSide | undefined;
  revealPanel: (side: PanelSide) => void;
}) {
  const { pathname = {} } = usePageNavigation() ?? {};
  const navigating = Object.values(pathname).some((v) => v !== undefined);
  switch (revealed) {
    case "center":
      return (
        <Fragment>
          <SidebarToolbarButton
            icon={
              navigating ? (
                <Spinner style={{ position: "relative", top: 1 }} />
              ) : (
                <FontAwesomeIcon icon={faBook} width={16} />
              )
            }
            onClick={() => revealPanel("left")}
          >
            <Trans>Chapters</Trans>
          </SidebarToolbarButton>
          <SidebarToolbarButton
            icon={<FontAwesomeIcon icon={faListUl} width={16} />}
            onClick={() => revealPanel("right")}
            style={{ flexDirection: "row-reverse" }}
          >
            <Trans>Sections</Trans>
          </SidebarToolbarButton>
        </Fragment>
      );
      break;
    default:
      return (
        <SidebarToolbarButton
          icon={<FontAwesomeIcon icon={faRightToBracket} />}
          onClick={() => revealPanel("center")}
          style={{ justifyContent: "center" }}
        >
          <Trans>Back to Page</Trans>
        </SidebarToolbarButton>
      );
  }
}

const ResizeHandle = styled(PanelResizeHandle)`
  position: relative;
  z-index: 500;
  width: 4px;
  padding: 0;
  margin: 0;
  transition: background-color 0.2s;

  &:hover,
  &[data-resize-handle-state="drag"] {
    background-color: #1677ff;
  }
`;

const SidebarToolbarContainer = styled.div`
  position: sticky;
  top: ${theme.dimensions.navbarHeight};
  z-index: 2;
  display: flex;
  flex-flow: row nowrap;
  gap: 0.8rem;
  align-items: center;
  justify-content: space-between;
  height: ${theme.dimensions.mobileToolbarHeight};
  padding: 0 0.8rem;
  background: white;
  box-shadow:
    0 1px 2px -2px rgb(0 0 0 / 9%),
    0 3px 6px 0 rgb(0 0 0 / 6%),
    0 5px 12px 4px rgb(0 0 0 / 3%);
`;

const SidebarToolbarButton = styled(Button).attrs({ type: "text", block: true })`
  justify-content: flex-start;
  height: 32px;
  padding: 0 11px;
  font-size: 16px;
  font-weight: 500;
  line-height: 100%;
  background: rgb(0 0 0 / 4%);
`;
