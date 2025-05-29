import { faChevronDown } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Select, Typography } from "antd";
import type { ComponentRef } from "react";
import { useRef, useState } from "react";
import { useNavigate } from "react-router";

import {
  getRepoExtras,
  useClientMeasurements,
  usePageNavigation,
  useRepoContent,
  useSiteContent,
} from "../app";
import { useThemeToken } from "../theme";

import { RepoLoading } from "./RepoLoading";
import { Spinner } from "./Spinner";

export function ProjectPicker() {
  const {
    path: { build: buildPath },
    versions,
    extras,
  } = useSiteContent();

  const {
    repo: { project },
  } = useRepoContent();

  const navigate = useNavigate();

  const navigation = usePageNavigation();

  const [search, setSearch] = useState("");

  const ref = useRef<ComponentRef<typeof Select>>(null);

  const resetSearch = () => {
    setSearch("");
    ref.current?.blur();
  };

  const { colors } = useThemeToken();

  const options = [...new Set(Object.keys(versions))]
    //
    .filter((repo) => !search || repo.toLowerCase().includes(search.toLowerCase()));

  options.sort((a, b) => {
    const r1 = getRepoExtras({ repo: a, lang: project.lang }, extras).displayOrder;
    const r2 = getRepoExtras({ repo: b, lang: project.lang }, extras).displayOrder;
    return r1 - r2 || a.localeCompare(b);
  });

  return (
    <Select<string>
      variant="filled"
      style={{ width: "100%", fontSize: 16, fontWeight: 500 }}
      suffixIcon={
        navigation?.pathname.repo ? (
          <Spinner style={{ color: colors.fg.link }} />
        ) : (
          <FontAwesomeIcon icon={faChevronDown} />
        )
      }
      getPopupContainer={(elem: HTMLElement) =>
        (elem.offsetParent as HTMLElement) || document.body
      }
      value={buildPath({ ...project, ref: undefined })}
      options={options.map((repo) => ({
        label: (
          <Typography.Text
            style={{
              fontSize: 16,
              fontWeight: 500,
              display: "inline-flex",
              alignItems: "center",
            }}
          >
            <span>
              {getRepoExtras({ repo, lang: project.lang }, extras).projectName}
            </span>
            <RepoLoading repo={repo} style={{ marginInlineStart: 4 }} />
          </Typography.Text>
        ),
        value: buildPath({
          repo,
          lang: project.lang,
          ref: undefined,
        }),
      }))}
      onChange={(path) => {
        navigate(path);
        resetSearch();
      }}
      onSearch={(input) => setSearch(input)}
      showSearch={!useClientMeasurements().smallScreen}
      filterOption={false}
      ref={ref}
    />
  );
}
