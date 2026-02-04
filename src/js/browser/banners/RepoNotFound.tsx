import { Trans, useLingui } from "@lingui/react/macro";
import { Fragment } from "react";
import { Helmet } from "react-helmet-async";

import { quoteUrl, useErrorEnum } from "../error";
import type { ErrorCause } from "../error";
import { RepoLoading } from "../layout/RepoLoading";

import { ErrorDetails, userInput } from "./ErrorDetails";

export function RepoNotFound({
  reason: { repo },
}: {
  reason: Extract<ErrorCause, { code: "repo-4xx" }>;
}) {
  const { t } = useLingui();
  return (
    <Fragment>
      <Helmet>
        <title>{t`Documentation not found`}</title>
      </Helmet>
      <ErrorDetails
        type="warning"
        title={
          <span>
            <Trans>Documentation not found</Trans>
            <RepoLoading
              style={{ position: "relative", top: 2, marginInlineStart: 6 }}
              repo
              ref_
              lang
            />
          </span>
        }
        error={useErrorEnum()}
      >
        <p style={userInput}>
          <Trans>
            Could not find documentation for <strong>{quoteUrl(repo)}</strong>.
          </Trans>
        </p>
        <p>
          <Trans>
            If you entered a web address, check it is correct. The content may have
            otherwise been moved or deleted.
          </Trans>
        </p>
      </ErrorDetails>
    </Fragment>
  );
}
