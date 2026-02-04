import { Trans, useLingui } from "@lingui/react/macro";
import { Typography } from "antd";
import leven from "leven";
import { Fragment } from "react";
import { Helmet } from "react-helmet-async";
import { useLocation } from "react-router";

import type { Project, SidebarItem } from "../../docs/types";
import {
  getSidebar,
  IntraLink,
  iterSidebar,
  useRepoContent,
  useRepoExtras,
  useSiteContent,
} from "../app";
import { quoteUrl } from "../error";
import type { ErrorCause } from "../error";
import { wordBreak } from "../page/whitespace";
import { useFullTextSearch } from "../search/client";

import { ErrorDetails, link, userInput } from "./ErrorDetails";

export function PageNotFound({
  reason: {
    path: { suffix },
  },
}: {
  reason: Extract<ErrorCause, { code: "page-404" }>;
}) {
  const { t } = useLingui();

  const {
    path: { parse: parsePath, build: buildPath },
  } = useSiteContent();

  const {
    repo: { project },
  } = useRepoContent();

  const { pathname } = useLocation();
  suffix ??= pathname;
  suffix ||= "";

  const versionName = quoteUrl(`${useRepoExtras(project).projectName} ${project.ref}`);

  const suggestions = closestPages({ project, suffix }) //
    .map(([, { key: suffix, title }]) => ({ suffix, title }));

  const search = useFullTextSearch({ query: suffix.replaceAll(/_+/g, " "), limit: 50 });

  if (search.results) {
    for (const item of search.results.pages.flatMap((page) => page.items)) {
      if (suggestions.length >= 10) {
        break;
      }
      let { suffix } = parsePath(item.document.url) ?? {};
      [suffix] = suffix?.split("?") ?? [];
      [suffix] = suffix?.split("#") ?? [];
      if (suffix === undefined || suggestions.some((s) => s.suffix === suffix)) {
        continue;
      }
      suggestions.push({ suffix, title: item.document.title });
    }
  }

  return (
    <Fragment>
      <Helmet>
        <title>{t`Page not found`}</title>
      </Helmet>
      <ErrorDetails type="warning" title={t`Page not found`}>
        <p style={userInput}>
          <Trans>
            The page <strong>{wordBreak(quoteUrl(suffix))}</strong> was not found in{" "}
            {versionName}.
          </Trans>
        </p>
        <p>
          <Trans>
            If you entered a web address, check it is correct. The content may have
            otherwise been moved or deleted.
          </Trans>
        </p>
        {suggestions.length ? (
          <Fragment>
            <p>
              <Trans>You may also be looking for:</Trans>
            </p>
            <ul style={{ margin: 0, paddingInlineStart: "1.5rem" }}>
              {suggestions.map(({ suffix, title }) => (
                <li key={suffix} style={{ marginBlock: 3 }}>
                  <IntraLink to={buildPath({ ...project, suffix })} style={link}>
                    {wordBreak(title)}
                  </IntraLink>
                  {suffix ? (
                    <Typography.Text type="secondary" style={{ marginInlineStart: 6 }}>
                      ({wordBreak(suffix)})
                    </Typography.Text>
                  ) : undefined}
                </li>
              ))}
            </ul>
          </Fragment>
        ) : null}
        <p>
          <IntraLink to={buildPath({ ...project, suffix: "" })} style={link}>
            <Trans>Click here to return to the home page of {versionName}</Trans>
          </IntraLink>
        </p>
        {!search.ready || search.searching ? (
          <p>
            <Typography.Text type="secondary">
              <Trans>Searching for more pages ...</Trans>
            </Typography.Text>
          </p>
        ) : null}
      </ErrorDetails>
    </Fragment>
  );

  function closestPages({ project, suffix }: { project: Project; suffix: string }) {
    const candidates: [number, SidebarItem][] = [];
    for (const item of iterSidebar(getSidebar(project))) {
      if (item.kind !== "doc") {
        continue;
      }
      const length = Math.max(suffix.length, item.key.length);
      const distance = leven(suffix, item.key) / length;
      candidates.push([distance, item]);
    }
    candidates.sort(([a], [b]) => a - b);
    return candidates.slice(0, 3);
  }
}
