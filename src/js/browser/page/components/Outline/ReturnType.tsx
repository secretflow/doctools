import type { PropsWithChildren } from "react";

import { wordBreak } from "../../whitespace";

export function ReturnType({ children }: PropsWithChildren) {
  return <em>{wordBreak(children)}</em>;
}
