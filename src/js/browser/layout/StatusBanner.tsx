import { faLock } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Alert, Divider, Typography } from "antd";
import { styled } from "styled-components";

import { theme } from "../theme";

export function StatusBanner() {
  return (
    <Warning
      banner
      message={
        <WarningMessage>
          <Typography.Text ellipsis>
            <Trans>SecretFlow Documentation</Trans>
          </Typography.Text>
          <InlineDivider />
          <Typography.Text
            style={{
              fontWeight: 600,
              textTransform: "uppercase",
              whiteSpace: "nowrap",
            }}
          >
            <FontAwesomeIcon icon={faLock} style={{ marginInlineEnd: "5px" }} />
            <Trans>For Preview Only</Trans>
          </Typography.Text>
        </WarningMessage>
      }
      type="warning"
      showIcon={false}
    />
  );
}

const Warning = styled(Alert)`
  position: sticky;
  top: 0;
  z-index: 50;
  flex: 0 0 auto;
  min-width: 0;
  height: ${theme.dimensions.navbarHeight};
  padding: 8px 20px;
  background: #ffc53d
    repeating-linear-gradient(
      135deg,
      transparent,
      transparent 32px,
      rgb(255 255 255 / 20%) 32px,
      rgb(255 255 255 / 20%) 64px
    );
  box-shadow:
    0 1px 2px -2px rgb(0 0 0 / 9%),
    0 3px 6px 0 rgb(0 0 0 / 6%),
    0 5px 12px 4px rgb(0 0 0 / 3%);
`;

const WarningMessage = styled.div`
  display: flex;
  gap: 0.5rem;
  align-items: center;
  justify-content: center;
  width: fit-content;
  min-width: 0;
  max-width: 100%;
  padding: 2px 1ch;
  margin: 0 auto;
  font-weight: 500;
  background-color: #ffffff80;
`;

const InlineDivider = styled(Divider).attrs({ type: "vertical" })`
  margin: 0;
`;
