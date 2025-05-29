import type { PropsWithChildren } from "react";

import { wordBreak } from "../../whitespace";

export function Prefix({ children }: PropsWithChildren) {
  return <span>{wordBreak(children)}</span>;
}
