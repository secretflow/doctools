import { Typography } from "antd";
import { toJsxRuntime } from "hast-util-to-jsx-runtime";
import { Fragment, useEffect, useState } from "react";
import { styled } from "styled-components";

import { runtime } from "../../../jsx";
import { breakpoint, theme } from "../../../theme";

import type { IntoWorker } from "./types";
import { colorReplacements } from "./types";

export function SourceCode({
  children: code,
  lang,
  className,
}: {
  children: string;
  lang: string;
  className?: string;
}) {
  return (
    <Container>
      <CopyButton copyable={{ text: code }} />
      <Backdrop className={className}>
        <Highlighted code={code} lang={lang} />
      </Backdrop>
    </Container>
  );
}

function Highlighted({ code, lang }: IntoWorker) {
  const fallback = () => (
    <pre>
      <code className="shiki-pending">{code}</code>
    </pre>
  );

  const [tree, setTree] = useState({ code, lang, tree: fallback() });

  useEffect(() => {
    (async () => {
      const { highlight } = await import("./highlight/server");
      const result = await highlight({ code, lang });
      const tree = toJsxRuntime(result.root, runtime);
      setTree({ tree, ...result });
    })();
  }, [code, lang]);

  if (tree.code === code) {
    return (
      <Fragment>
        <Language>{tree.lang}</Language>
        {tree.tree}
      </Fragment>
    );
  } else {
    return fallback();
  }
}

const CopyButton = styled(Typography.Text)`
  position: absolute;
  top: 6px;
  right: 6px;
  line-height: 1;
  opacity: 0;
  transition: opacity 0.2s;

  .ant-typography-copy {
    width: 32px;
    height: 32px;
    font-size: 16px;
    color: hsl(239.9deg 5.73% 64.3%);
    background-color: ${colorReplacements["#fff"]};
  }
`;

const Language = styled.code`
  position: absolute;
  top: 14px;
  right: 40px;
  font-size: 0.7rem;
  line-height: 1;
  color: rgb(140 140 140) !important;
  text-align: end;
  user-select: none;
  opacity: 0;
  transition: opacity 0.2s;

  ${breakpoint("mobileWidth")} {
    display: none;
  }
`;

const Backdrop = styled.div`
  width: 100%;
  padding: 12px 18px;
  overflow: auto;
  font-size: 11pt;
  line-height: 1.6;
  background-color: ${colorReplacements["#fff"]};
  border-radius: 6px;

  pre {
    width: max-content;

    code {
      color: ${theme.colors.fg.default};
    }
  }

  ${breakpoint("mobileWidth")} {
    line-height: 1.5;
  }
`;

const Container = styled.div`
  position: relative;
  max-width: 100%;

  &:hover {
    ${Language} {
      opacity: 0.8;
    }

    ${CopyButton} {
      opacity: 1;
    }
  }
`;

SourceCode.selector = Backdrop;
