import type { PropsWithChildren } from "react";

import { wordBreak } from "../../whitespace";

export function Parameter({ children }: PropsWithChildren) {
  return <em>{wordBreak(children)}</em>;
}
