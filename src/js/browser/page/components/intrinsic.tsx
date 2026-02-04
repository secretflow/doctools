import { faExternalLink } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import type { ComponentProps } from "react";

import { permalink, highlighted } from "../anchoring";
import { wordBreak } from "../whitespace";

const h1 = permalink(highlighted("h1"));
const h2 = permalink(highlighted("h2"));
const h3 = permalink(highlighted("h3"));
const h4 = permalink(highlighted("h4"));
const h5 = permalink(highlighted("h5"));
const h6 = permalink(highlighted("h6"));

export { h1, h2, h3, h4, h5, h6 };

const Anchor = highlighted("a");

export function code({ children, ...props }: ComponentProps<"code">) {
  return <code {...props}>{wordBreak(children)}</code>;
}

export function a({ children, target, ...props }: ComponentProps<"a">) {
  return (
    <Anchor {...props} target={target} rel="noopener noreferrer nofollow">
      {wordBreak(children)}
      {target === "_blank" ? (
        <FontAwesomeIcon
          icon={faExternalLink}
          style={{
            width: 10,
            height: 10,
            marginInlineStart: 6,
            marginInlineEnd: 3,
            verticalAlign: "middle",
          }}
        />
      ) : null}
    </Anchor>
  );
}
