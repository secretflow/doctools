import { faChevronDown } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useLingui } from "@lingui/react/macro";
import { Select, Typography } from "antd";
import type { ComponentProps, ComponentRef } from "react";
import { useRef, useState } from "react";
import { useNavigate } from "react-router";

import type { Project } from "../../docs/types";
import {
  useClientMeasurements,
  usePageNavigation,
  usePathPatcher,
  useRepoContent,
  useRepoExtras,
  useSiteContent,
} from "../app";
import { useThemeToken } from "../theme";

import { RepoLoading } from "./RepoLoading";
import { Spinner } from "./Spinner";

export function VersionPicker() {
  const { t } = useLingui();

  const { versions } = useSiteContent();

  const {
    repo: { project },
  } = useRepoContent();

  const { versionFilter } = useRepoExtras(project);

  const navigate = useNavigate();

  const patched = usePathPatcher();

  const navigation = usePageNavigation();

  const [search, setSearch] = useState("");

  const ref = useRef<ComponentRef<typeof Select>>(null);

  const resetSearch = () => {
    setSearch("");
    ref.current?.blur();
  };

  const { colors } = useThemeToken();

  const options = (() => {
    const { tags, head, rest } = versions[project.repo];

    const getOption = ({ ref }: Pick<Project, "ref">) => {
      if (search && !ref.toLocaleLowerCase().includes(search.toLocaleLowerCase())) {
        return [];
      }
      const tag =
        tags.find(({ raw }) => ref === raw) ??
        head.find(({ raw }) => ref === raw) ??
        rest.find(({ raw }) => ref === raw);
      if (tag && !versionFilter(tag)) {
        return [];
      }
      const value = patched({ ref });
      return [
        {
          value,
          label: (
            <Typography.Text
              style={{
                fontSize: 16,
                fontWeight: 500,
                display: "inline-flex",
                alignItems: "center",
              }}
            >
              <span>{ref}</span>
              <RepoLoading ref_={ref} style={{ marginInlineStart: 4 }} />
            </Typography.Text>
          ),
        },
      ];
    };

    const sorted = [...tags];
    sorted.reverse();

    const releases = sorted
      .filter((t) => t.label === "stable")
      .flatMap(({ raw: ref }) => getOption({ ref }));

    const prereleases = sorted
      .filter((t) => t.label === "alpha" || t.label === "beta" || t.label === "rc")
      .flatMap(({ raw: ref }) => getOption({ ref }));

    const nightlies = sorted
      .filter((t) => t.label === "dev")
      .flatMap(({ raw: ref }) => getOption({ ref }));

    const options: ComponentProps<typeof Select>["options"] = [];

    if (releases.length) {
      options.push({
        label: t`releases`,
        options: releases,
      });
    }

    if (prereleases.length) {
      options.push({
        label: t`prereleases`,
        options: prereleases,
      });
    }

    if (nightlies.length) {
      options.push({
        label: t`nightlies`,
        options: nightlies,
      });
    }

    if (head.length) {
      options.push({
        label: "HEAD",
        options: head.flatMap(({ raw: ref }) => getOption({ ref })),
      });
    }

    if (rest.length) {
      options.push({
        label: t`other versions`,
        options: rest.flatMap(({ raw: ref }) => getOption({ ref })),
      });
    }

    return options;
  })();

  return (
    <Select<string>
      variant="filled"
      style={{ width: "100%" }}
      suffixIcon={
        navigation?.pathname.ref ? (
          <Spinner style={{ color: colors.fg.link }} />
        ) : (
          <FontAwesomeIcon icon={faChevronDown} />
        )
      }
      getPopupContainer={(elem: HTMLElement) =>
        (elem.offsetParent as HTMLElement) || document.body
      }
      value={patched({})}
      options={options}
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
