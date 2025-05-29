import { useMDXComponents } from "@mdx-js/react";

import { createDocumentationSite } from "./app";
import type {
  Environment,
  ExternalRouter,
  MatchedPath,
  PathFormat,
  RepoExtras,
  RepoLoader,
  SlottedComponent,
} from "./app";
import { isSpuriousLocale } from "./i18n";
import { fuzzyPath, npmLoader } from "./loaders";
import type {
  FuzzyPathOptions,
  NPMLoaderOptions,
  NPMProvider,
  NPMRequest,
  NPMVersion,
} from "./loaders";

export {
  createDocumentationSite,
  fuzzyPath,
  isSpuriousLocale,
  npmLoader,
  useMDXComponents,
};
export type {
  Environment,
  ExternalRouter,
  FuzzyPathOptions,
  MatchedPath,
  NPMLoaderOptions,
  NPMProvider,
  NPMRequest,
  NPMVersion,
  PathFormat,
  RepoExtras,
  RepoLoader,
  SlottedComponent,
};

export type * from "../docs/types";
