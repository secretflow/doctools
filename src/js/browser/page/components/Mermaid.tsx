import { Trans } from "@lingui/react/macro";
import { Flex, Typography } from "antd";
import mermaid from "mermaid";
import { Fragment, useEffect, useRef, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import { styled } from "styled-components";

import { Loading } from "../../layout/Loading";
import { useThemeToken } from "../../theme";

const Container = styled.div<{ align?: "flex-start" | "center" | "flex-end" }>`
  display: flex;
  flex-flow: column nowrap;
  align-items: ${({ align }) => align || "center"};
  margin-block: 1rem;
`;

mermaid.initialize({ startOnLoad: false });

// looks like mermaid isn't thread-safe
const queue: (() => Promise<void>)[] = [];

export function Mermaid({
  code,
  align = "center",
}: {
  code?: string;
  align?: "left" | "center" | "right";
}) {
  const ref = useRef<HTMLDivElement>(null);

  const [fallback, setFallback] = useState<ReactNode>(
    <Flex vertical gap={6}>
      <Loading paragraph={{ rows: 1 }} />
      <Typography.Text type="secondary">
        <Trans>Loading Mermaid diagram ...</Trans>
      </Typography.Text>
    </Flex>,
  );

  useEffect(() => {
    if (!ref.current) {
      return;
    }
    const elem = ref.current;
    const render = async () => {
      await mermaid.run({ nodes: [elem] });
      setFallback(null);
      // delete this task from the task queue
      queue.splice(queue.findIndex(render), 1);
      // if there are more tasks, run the next one
      queue.shift()?.();
    };
    queue.push(render);
    // if this is the only task, run it
    if (queue.length === 1) {
      render();
    }
  }, []);

  const { fonts } = useThemeToken();

  let alignItems: CSSProperties["alignItems"];
  switch (align) {
    case "left":
      alignItems = "flex-start";
      break;
    case "center":
      alignItems = "center";
      break;
    case "right":
      alignItems = "flex-end";
      break;
    default:
      alignItems = "center";
      break;
  }

  return (
    <Fragment>
      {fallback}
      <Container
        ref={ref}
        className="mermaid"
        style={
          fallback
            ? {
                alignItems: "flex-start",
                color: "rgba(0,0,0,0.45)",
                fontFamily: fonts.monospace,
                margin: 0,
                textAlign: "start",
                whiteSpace: "pre",
              }
            : {
                alignItems,
              }
        }
      >
        {code}
      </Container>
    </Fragment>
  );
}
