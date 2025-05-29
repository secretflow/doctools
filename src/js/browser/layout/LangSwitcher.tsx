import { faChevronDown, faLanguage } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useLingui } from "@lingui/react";
import { Select } from "antd";
import { useNavigate } from "react-router";
import { styled } from "styled-components";

import {
  useProjectLanguages,
  getSupportedLanguages,
  usePageNavigation,
  useRepoContent,
  usePathPatcher,
} from "../app";
import { printLocaleNameIn } from "../i18n";
import { useThemeToken } from "../theme";

import { Spinner } from "./Spinner";

export function LangSwitcher() {
  const navigate = useNavigate();

  const patched = usePathPatcher();

  const {
    repo: {
      project: { repo, ref, lang },
    },
  } = useRepoContent();

  const translated = useProjectLanguages({ repo, ref });

  const navigation = usePageNavigation();

  const { i18n } = useLingui();

  const { colors } = useThemeToken();

  return (
    <LangSwitcherSelect
      variant="filled"
      style={{ height: 32, minWidth: 0 }}
      suffixIcon={
        navigation?.pathname.lang ? (
          <Spinner style={{ color: colors.fg.link }} />
        ) : (
          <FontAwesomeIcon icon={faChevronDown} />
        )
      }
      getPopupContainer={(node) => node.parentElement || document.body}
      value={lang}
      onChange={(lang) => navigate(patched({ lang }))}
      options={getSupportedLanguages(i18n).map((lang) => ({
        value: lang,
        label: (
          <span style={{ fontWeight: 500, fontSize: "1rem" }}>
            <LangSwitcherIcon icon={faLanguage} />
            {printLocaleNameIn(lang)}
          </span>
        ),
        disabled: !translated.includes(lang),
      }))}
    />
  );
}

const LangSwitcherIcon = styled(FontAwesomeIcon)`
  margin-inline-end: 1ch;
`;

const LangSwitcherSelect = styled(Select<string>)`
  .ant-select-selector .anticon {
    display: none;
  }

  .ant-select-item-option-content ${LangSwitcherIcon} {
    display: none;
  }
`;
