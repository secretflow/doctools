import type { PropsWithChildren } from "react";

import { Term } from "./Term";

export const DefinitionList = ({ children }: PropsWithChildren) => children;

Object.assign(DefinitionList, { Term });
