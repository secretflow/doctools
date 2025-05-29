import { msg, plural } from "@lingui/core/macro";
import { Trans, useLingui } from "@lingui/react/macro";
import type { InputRef } from "antd";
import { ConfigProvider, Input, List, Modal, Progress, Typography } from "antd";
import { debounce } from "lodash";
import type { ReactNode, RefObject } from "react";
import { useEffect, useRef, useState } from "react";
import InfiniteScroll from "react-infinite-scroll-component";
import { useLocation } from "react-router";
import { styled } from "styled-components";

import type { SearchResult } from "../../search";
import {
  IntraLink,
  useRepoContent,
  useRepoExtras,
  useClientMeasurements,
} from "../app";
import { Spinner } from "../layout/Spinner";

import { useFullTextSearch } from "./client";

const Input2 = styled(Input)`
  font-size: 1.4rem;
  font-weight: 400;
  border-bottom: 2px solid #ebebeb;
  border-radius: 0;

  &:focus-within,
  &:active {
    border-bottom: 2px solid #1677ff;
  }
`;

const SearchResultTitle = styled(Typography.Paragraph)`
  font-size: 1.05rem;
  font-weight: 600;
`;

const SearchResultListItem = styled(List.Item)`
  display: flex;
  flex-flow: column nowrap;
  align-items: stretch;
  min-width: 0%;

  &:hover ${SearchResultTitle} {
    color: #1677ff;
    text-decoration: underline;
  }
`;

function useDocumentSearch({ scrollParent }: { scrollParent: string }) {
  const { t, i18n } = useLingui();

  const [input, setInput] = useState("");
  const [query, setQuery] = useState("");

  const inputRef = useRef<InputRef>(null);
  const inputComposition = useRef(false);

  const updateQuery = debounce((query: string) => setQuery(query), 200);

  const resetQuery = useCurrent(() => {
    inputComposition.current = false;
    setInput("");
    setQuery("");
  });

  const { results, searching, next, ready, loaded } = useFullTextSearch({
    query,
    limit: 25,
  });

  const {
    repo: { project },
  } = useRepoContent();

  const received = results?.pages.flatMap((p) => p.items) ?? [];

  const total = results?.pages?.[0]?.totalCount;

  const title = `${useRepoExtras(project).projectName} ${project.ref}`;

  return {
    ready,
    results,
    searching,

    query,
    updateQuery,
    resetQuery,

    inputRef,

    acceptInput: (
      <Input2
        ref={inputRef}
        size="large"
        variant="borderless"
        value={input}
        onChange={(e) => {
          setInput(e.target.value);
          if (!inputComposition.current) {
            updateQuery(e.target.value);
          }
        }}
        onCompositionStart={() => {
          inputComposition.current = true;
        }}
        onCompositionEnd={() => {
          inputComposition.current = false;
          if (inputRef.current?.input?.value) {
            updateQuery(inputRef.current.input.value);
          }
        }}
        allowClear
        placeholder={
          !ready ? t`Downloading search index ...` : t`Search in ${title} ...`
        }
        prefix={!ready ? <SearchIndicatorPrefix loaded={loaded} /> : null}
        suffix={!ready ? <SearchIndicatorSuffix loaded={loaded} /> : null}
      />
    ),

    resultList: query ? (
      <InfiniteScroll
        dataLength={received.length}
        hasMore={Boolean(total && total > received.length)}
        loader={
          <Typography.Text type="secondary">
            <Trans>Loading more results ...</Trans>
          </Typography.Text>
        }
        next={next}
        scrollableTarget={scrollParent}
      >
        <List<SearchResult>
          dataSource={received}
          rowKey={(d) => d.id}
          renderItem={(d) => (
            <SearchResultListItem style={{ alignItems: "stretch" }}>
              <SearchResultTitle style={{ margin: "3px 0px" }}>
                <IntraLink to={d.document.url} style={{ color: "inherit" }}>
                  {d.document.title}
                </IntraLink>
              </SearchResultTitle>
              <Typography.Paragraph
                type="secondary"
                style={{ fontSize: "0.8rem", margin: 0 }}
                ellipsis={{ rows: 2 }}
              >
                {d.document.content || d.document.url}
              </Typography.Paragraph>
            </SearchResultListItem>
          )}
        />
      </InfiniteScroll>
    ) : null,

    footer: (
      <Typography.Text type="secondary" style={{ fontSize: "0.8rem" }}>
        {(() => {
          if (ready && query && results) {
            return i18n._(
              msg`${plural(total ?? 0, { one: "# result", other: "# results" })}`,
            );
          }
          return null;
        })()}
      </Typography.Text>
    ),
  };
}

function SearchIndicatorPrefix(...props: Parameters<typeof useSearchIndicator>) {
  return useSearchIndicator(...props).prefix;
}

function SearchIndicatorSuffix(...props: Parameters<typeof useSearchIndicator>) {
  const { suffix } = useSearchIndicator(...props);
  const { smallScreen } = useClientMeasurements();
  if (smallScreen) {
    return null;
  } else {
    return suffix;
  }
}

function useSearchIndicator({
  loaded,
}: Pick<ReturnType<typeof useFullTextSearch>, "loaded">) {
  const [progress, setProgress] = useState(loaded());

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress(loaded());
    }, 300);
    return () => clearInterval(interval);
  }, [loaded]);

  const {
    received,
    sizeHint: { upper: total },
  } = progress ?? { sizeHint: {} };

  const indeterminate = !received || !total;

  if (indeterminate) {
    let displayedSize: string | undefined;
    if (received) {
      if (received > 1e6) {
        displayedSize = `${Math.round(received / 1e5) / 10} MB`;
      } else if (received > 1e3) {
        displayedSize = `${Math.round(received / 1e2) / 10} KB`;
      } else {
        displayedSize = `${Math.round(received / 1e-1) / 10} B`;
      }
    }
    return {
      prefix: (
        <Spinner
          style={{
            marginInlineEnd: "0.5ch",
            color: "rgb(0, 0, 0, .25)",
          }}
        />
      ),
      suffix: displayedSize ? (
        <Typography.Text
          style={{ height: 32, fontSize: "0.9em", color: "rgb(0, 0, 0, .25)" }}
        >
          ... {displayedSize}
        </Typography.Text>
      ) : null,
    };
  } else {
    const percentage = Math.round((received / total) * 1000) / 10;
    return {
      prefix: (
        <Progress
          type="circle"
          size={20}
          percent={percentage}
          style={{ marginInlineEnd: "0.5ch" }}
        />
      ),
      suffix: null,
    };
  }
}

const SCROLL_PARENT_ID = "search-result-list-parent";

export function DesktopSearch({
  createTrigger,
}: {
  createTrigger: (inject: { onOpen: () => void }) => ReactNode;
}) {
  const [visible, setVisible] = useState(false);

  const location = useLocation();

  const { acceptInput, inputRef, ready, resetQuery, footer, resultList } =
    useDocumentSearch({ scrollParent: SCROLL_PARENT_ID });

  const makeInvisible = useCurrent(() => {
    resetQuery.current?.();
    setVisible(false);
  });

  useEffect(() => {
    void location;
    const dispose = makeInvisible.current;
    return () => dispose?.();
  }, [location, makeInvisible, resetQuery]);

  const onOpen = () => setVisible(true);

  const { smallScreen } = useClientMeasurements();

  return (
    <>
      {createTrigger({ onOpen })}
      <ConfigProvider theme={{ token: { motion: false } }}>
        <Modal
          title={
            <div
              style={{
                display: "flex",
                flexFlow: "row nowrap",
                alignItems: "flex-end",
                position: "relative",
                boxSizing: "border-box",
                gap: "0.5rem",
                marginBottom: "1rem",
              }}
            >
              {acceptInput}
            </div>
          }
          open={visible}
          footer={
            <div
              style={{
                display: "flex",
                gap: "1rem",
                alignItems: "baseline",
                justifyContent: "flex-end",
              }}
            >
              {footer}
            </div>
          }
          closable={false}
          afterOpenChange={(open) => {
            if (open) {
              inputRef.current?.focus();
            }
          }}
          onCancel={() => makeInvisible.current?.()}
          width={smallScreen ? "calc(100vw - 16px)" : "60vw"}
          style={{ top: smallScreen ? 0 : 72, maxWidth: 1024 }}
        >
          <div
            id={SCROLL_PARENT_ID}
            style={{
              maxHeight: smallScreen ? "calc(100vh - 160px)" : "calc(100vh - 288px)",
              overflowY: "auto",
            }}
          >
            {ready ? resultList : null}
          </div>
        </Modal>
      </ConfigProvider>
    </>
  );
}

function useCurrent<T>(value: T): RefObject<T> {
  const ref = useRef(value);
  ref.current = value;
  return ref;
}
