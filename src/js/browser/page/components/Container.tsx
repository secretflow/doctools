import { faLightbulb } from "@fortawesome/free-regular-svg-icons";
import {
  faCircleInfo,
  faExclamationTriangle,
  faXmarkCircle,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import type { ComponentProps, PropsWithChildren, ReactNode } from "react";
import { styled } from "styled-components";

import { breakpoint, theme } from "../../theme";

const BaseContainer = styled.aside`
  display: flex;
  flex-flow: column nowrap;
  gap: 0.5rem;
  align-items: stretch;
  padding: 18px 40px;
  border-radius: 6px;

  ${breakpoint("mobileWidth")} {
    padding: 1rem;
  }

  ${"@container (width < 1024px)"} {
    padding: 1rem;
  }
`;

const BaseTitle = styled.h4`
  position: relative;
  margin: 0;
  margin-block-end: 0.4rem;
  font-family: ${theme.fonts.sansSerif};
  font-size: 15px;
  color: inherit;
`;

const BaseIcon = styled(FontAwesomeIcon)`
  position: absolute;
  top: 0;
  left: -25px;
  width: 16px;
  height: 100%;
  color: inherit;

  ${breakpoint("mobileWidth")} {
    position: static;
    margin-inline-end: 0.8ch;
  }

  ${"@container (width < 1024px)"} {
    position: static;
    margin-inline-end: 0.8ch;
  }
`;

const InfoContainer = styled(BaseContainer)`
  color: #496a99;
  background-color: #ecf4ff;
`;

const infoIcon = <BaseIcon icon={faCircleInfo} />;

const TipContainer = styled(BaseContainer)`
  color: #357047;
  background-color: #dff8e7;
`;

const tipIcon = <BaseIcon icon={faLightbulb} />;

const WarningContainer = styled(BaseContainer)`
  color: #7e6224;
  background-color: #fff3da;
`;

const warningIcon = <BaseIcon icon={faExclamationTriangle} />;

const DangerContainer = styled(BaseContainer)`
  color: #955359;
  background-color: #fdf4f5;
`;

const dangerIcon = <BaseIcon icon={faXmarkCircle} />;

export function Container({
  type,
  title,
  icon,
  children,
  ...props
}: PropsWithChildren<
  {
    type?: "info" | "success" | "warning" | "error";
    title?: ReactNode;
    icon?: ReactNode;
  } & Omit<ComponentProps<"aside">, "title">
>) {
  if (title === undefined) {
    switch (type) {
      case "warning":
        title = <Trans>Warning</Trans>;
        break;
      case "error":
        title = <Trans>Danger</Trans>;
        break;
      case "success":
        title = <Trans>Tip</Trans>;
        break;
      default:
        title = <Trans>Note</Trans>;
        break;
    }
  }

  let Container: (props: ComponentProps<"aside">) => ReactNode;

  switch (type) {
    case "warning":
      Container = WarningContainer;
      break;
    case "error":
      Container = DangerContainer;
      break;
    case "success":
      Container = TipContainer;
      break;
    default:
      Container = InfoContainer;
      break;
  }

  if (!icon) {
    switch (type) {
      case "warning":
        icon = warningIcon;
        break;
      case "error":
        icon = dangerIcon;
        break;
      case "success":
        icon = tipIcon;
        break;
      default:
        icon = infoIcon;
        break;
    }
  }

  return (
    <Container {...props}>
      <BaseTitle>
        {icon}
        {title}
      </BaseTitle>
      {children}
    </Container>
  );
}

Container.selector = BaseContainer;

Container.Icon = BaseIcon;
