import { faPython } from "@fortawesome/free-brands-svg-icons";
import { faExternalLink } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Divider, Typography } from "antd";
import { matchPath } from "react-router";

import { usePartialPageContent } from "../app";

export function IsNotebook() {
  const { page: { content } = {} } = usePartialPageContent();

  const fetcher = async () => {
    const url = content?.frontmatter?.git_download_url;

    if (!url?.endsWith(".ipynb")) {
      return null;
    }

    if (!URL.canParse(url)) {
      return null;
    }

    const link = new URL(url);

    const match = matchPath("/:owner/:repo/raw/:ref/*", link.pathname);

    if (!match) {
      return;
    }

    const fileName = match.params["*"];

    const { owner, repo, ref } = match.params;

    const downloadURL = `https://raw.githubusercontent.com/${owner}/${repo}/${ref}/${fileName}`;

    const res = await fetch(downloadURL, { redirect: "follow" });

    return res.blob();
  };

  const { refetch, isFetching } = useQuery({
    queryFn: fetcher,
    queryKey: [content?.frontmatter?.git_download_url],
    enabled: false,
  });

  const url = content?.frontmatter?.git_download_url;

  if (!url?.endsWith(".ipynb")) {
    return null;
  }

  const fileName = url.split("/").pop();

  return (
    <Alert
      banner
      type="success"
      icon={<FontAwesomeIcon icon={faPython} width="14px" />}
      message={
        <Typography.Text>
          <Trans>This is a Jupyter Notebook</Trans>
          <Divider type="vertical" />
          <Button
            type="link"
            onClick={() => {
              if (isFetching) {
                return;
              }
              refetch().then((res) => {
                if (!res?.data) {
                  return;
                }
                const url = URL.createObjectURL(res.data);
                const a = document.createElement("a");
                a.href = url;
                a.download = fileName || "notebook.ipynb";
                a.click();
              });
            }}
            style={{
              height: "initial",
              lineHeight: "1em",
              padding: 0,
              display: "inline-block",
              cursor: isFetching ? "progress" : "pointer",
            }}
          >
            <Trans>Download</Trans>
          </Button>
          <Divider type="vertical" />
          <Typography.Link
            href={content?.frontmatter?.git_origin_url}
            target="_blank"
            style={{ display: "inline-block" }}
          >
            <Trans>Open on GitHub</Trans>{" "}
            <FontAwesomeIcon icon={faExternalLink} fontSize={10} />
          </Typography.Link>
          <Divider type="vertical" />
          <Typography.Link
            href="https://jupyterlab.readthedocs.io/en/latest/getting_started/installation.html"
            target="_blank"
            type="secondary"
            style={{ display: "inline-block" }}
          >
            <Trans>How to use</Trans>{" "}
            <FontAwesomeIcon icon={faExternalLink} fontSize={10} />
          </Typography.Link>
        </Typography.Text>
      }
    />
  );
}
