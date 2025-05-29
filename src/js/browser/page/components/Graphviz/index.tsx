import type { ComponentProps } from "react";
import { Suspense, lazy } from "react";

import { Loading } from "../../../layout/Loading";

const GraphVizLazy = lazy(() =>
  import("./Graphviz").then(({ Graphviz }) => ({ default: Graphviz })),
);

export function Graphviz(props: ComponentProps<typeof GraphVizLazy>) {
  return (
    <Suspense fallback={<Loading />}>
      <GraphVizLazy {...props} />
    </Suspense>
  );
}
