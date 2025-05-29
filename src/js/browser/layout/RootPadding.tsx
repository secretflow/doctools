import type { PropsWithChildren } from "react";

export function RootPadding({ children }: Required<PropsWithChildren>) {
  return <div style={{ padding: 16, maxWidth: 800 }}>{children}</div>;
}
