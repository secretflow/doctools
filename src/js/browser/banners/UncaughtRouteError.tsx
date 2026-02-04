import { Trans, useLingui } from "@lingui/react/macro";
import { Typography } from "antd";
import { Fragment } from "react";
import type { ReactNode } from "react";
import { Helmet } from "react-helmet-async";

import { useErrorEnum } from "../error";
import { RepoLoading } from "../layout/RepoLoading";

import { ErrorDetails, link } from "./ErrorDetails";

export function UncaughtRouteError() {
  const { t } = useLingui();

  const error = useErrorEnum();

  let reason: ReactNode = (
    <Trans>
      The application encountered an unexpected error. We apologize for the
      inconvenience.
    </Trans>
  );

  switch (error?.cause.code) {
    case "http":
      reason = (
        <Trans>
          Received an HTTP {error.cause.res.status} error while downloading page
          content.
        </Trans>
      );
      break;
    case "unknown": {
      const { err } = error.cause;
      if (
        err instanceof TypeError &&
        /fetch|importing|imported|module|networkerror|load failed/i.test(err.message)
      ) {
        reason = (
          <Trans>Network problems occurred while downloading page content.</Trans>
        );
      }
      break;
    }
  }

  return (
    <Fragment>
      <Helmet>
        <title>{t`Could not display this page`}</title>
      </Helmet>
      <ErrorDetails
        type="error"
        error={error}
        title={
          <span>
            <Trans>Could not display this page</Trans>
            <RepoLoading
              style={{ position: "relative", top: 2, marginInlineStart: 6 }}
              repo
              ref_
              lang
            />
          </span>
        }
      >
        <p>{reason}</p>
        <p>
          <Trans>
            You may{" "}
            <Typography.Link href={window.location.href} style={link}>
              refresh this page
            </Typography.Link>
            , or try again later.
          </Trans>
        </p>
      </ErrorDetails>
    </Fragment>
  );
}
