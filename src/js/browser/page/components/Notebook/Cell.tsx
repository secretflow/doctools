import { styled } from "styled-components";

import { breakpoint } from "../../../theme";

export const Cell = styled.section`
  display: grid;
  grid-template-columns: minmax(80px, max-content) minmax(0, auto);
  gap: 0.5rem 12px;
  align-items: baseline;
  padding: 0.5rem 0;

  ${breakpoint("mobileWidth")} {
    grid-template-columns: minmax(0, 1fr);
    padding: 0;
  }
`;
