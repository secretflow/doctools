import { faSadTear } from "@fortawesome/free-regular-svg-icons";
import { Trans } from "@lingui/react/macro";
import type { ComponentProps, CSSProperties } from "react";
import { styled } from "styled-components";

import { Container } from "../page/components/Container";
import { theme } from "../theme";

export function ErrorDetails({
  error,
  title,
  type,
  children,
  icon,
  ...props
}: {
  error?: unknown;
} & Required<Pick<ContainerProps, "type" | "title">> &
  ContainerProps) {
  if (!icon && type === "error") {
    icon = <Container.Icon icon={faSadTear} />;
  }
  return (
    <ErrorContainer type={type} title={title} icon={icon} {...props}>
      <Flexbox>
        {children}
        {error ? (
          <div>
            <details>
              <summary>
                <Trans>Technical info</Trans>
              </summary>
              <pre
                style={{
                  marginTop: 8,
                  marginBottom: 0,
                  fontSize: "0.8rem",
                }}
                role="complementary"
              >
                <code>{error.toString()}</code>
              </pre>
            </details>
          </div>
        ) : null}
      </Flexbox>
    </ErrorContainer>
  );
}

const ErrorContainer = styled(Container)`
  font-family: ${theme.fonts.sansSerif};
  font-size: 14px;
  line-height: 1.4;

  h4 {
    margin: 0;
  }

  p {
    margin: 0;
  }

  pre {
    padding: 0.4em 0.6em;
    font-family: ${theme.fonts.monospace};
    word-wrap: break-word;
    white-space: pre-wrap;
    background: rgb(150 150 150 / 10%);
    border: 1px solid rgb(100 100 100 / 20%);
    border-radius: 3px;
  }

  .ant-typography {
    font-family: ${theme.fonts.sansSerif};
    line-height: 1.4;

    &:not(a, .ant-typography-secondary) {
      color: inherit;
    }
  }
`;

const Flexbox = styled.div`
  display: flex;
  flex-flow: column nowrap;
  gap: 8px;
  align-items: stretch;
  min-width: 0;
`;

type ContainerProps = ComponentProps<typeof ErrorContainer>;

export const link = {
  textDecoration: "underline",
  textDecorationStyle: "solid",
  textDecorationThickness: 1,
  textUnderlineOffset: 4,
  color: theme.colors.fg.link,
} satisfies CSSProperties;

export const userInput = {
  overflow: "hidden",
  textOverflow: "ellipsis",
} satisfies CSSProperties;
