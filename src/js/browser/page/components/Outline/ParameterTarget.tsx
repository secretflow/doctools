import type { PropsWithChildren } from "react";

import { wordBreak } from "../../whitespace";

export function ParameterTarget({ children }: PropsWithChildren) {
  return <strong>{wordBreak(children)}</strong>;
}
