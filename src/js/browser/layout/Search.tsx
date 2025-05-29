import { faSearch } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Trans } from "@lingui/react/macro";
import { Button } from "antd";
import { styled } from "styled-components";

import { DesktopSearch } from "../search";

export function Search() {
  return (
    <DesktopSearch
      createTrigger={({ onOpen }) => (
        <SearchButton
          icon={<FontAwesomeIcon icon={faSearch} />}
          type="text"
          onClick={() => onOpen()}
        >
          <Trans>Search</Trans>
        </SearchButton>
      )}
    />
  );
}

const SearchButton = styled(Button)`
  justify-content: flex-start;
  height: 32px;
  padding: 0 11px;
  font-size: 16px;
  font-weight: 500;
  line-height: 100%;
  background: rgb(0 0 0 / 4%);
`;
