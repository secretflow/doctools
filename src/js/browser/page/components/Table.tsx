import { useEffect, useRef } from "react";
import type { ComponentProps } from "react";
import { styled } from "styled-components";

import { theme } from "../../theme";

const TableOuter = styled.div`
  transform: translate(0, 0);
`;

const TableScroll = styled.div`
  overflow: auto visible;

  h1 {
    font-size: 1.5rem;
  }

  h2 {
    font-size: 1.2rem;
  }

  h3,
  h4,
  h5,
  h6 {
    font-size: 1rem;
  }

  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    font-weight: 600;
  }

  p {
    margin: 0;
  }

  &::before,
  &::after {
    position: fixed;
    top: 0;
    bottom: 0;
    z-index: 1;
    display: block;
    width: 6px;
    pointer-events: none;
    content: "";
  }

  &[data-overflow-left="true"]::before {
    left: 0;
    background-image: linear-gradient(to right, rgb(0 0 0 / 10%), rgb(0 0 0 / 0%));
  }

  &[data-overflow-right="true"]::after {
    right: 0;
    background-image: linear-gradient(to left, rgb(0 0 0 / 10%), rgb(0 0 0 / 0%));
  }
`;

const TableInner = styled.table`
  width: max-content;
  max-width: 64rem;
  word-break: normal;
  empty-cells: show;
  border-spacing: 0;
  border-collapse: collapse;
  border: 1px solid ${theme.colors.fg.container};

  @media screen and (width >= 1024px) {
    width: max-content;
    max-width: 52rem;
  }

  @media screen and (width >= 1440px) {
    width: auto;
    max-width: 100%;
  }

  thead {
    word-break: keep-all;
  }

  tbody {
    overflow: auto;

    tr {
      transition: all 0.3s;

      &:hover {
        background: rgb(60 90 100 / 4%);
      }
    }
  }

  th,
  td {
    padding: 6px 12px;
    text-align: left;
    border: 1px solid ${theme.colors.fg.container};
  }

  th {
    font-weight: 500;
    white-space: nowrap;
    background-color: ${theme.colors.bg.container};
  }

  td {
    vertical-align: top;
  }

  p {
    margin: 0;
  }
`;

export function Table(props: ComponentProps<"table">) {
  const outer = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const { current: elem } = outer;
    if (!elem) {
      return;
    }
    if (elem.clientWidth >= elem.scrollWidth) {
      return;
    }
    const listener = () => {
      elem.dataset["overflowLeft"] = String(elem.scrollLeft > 0);
      elem.dataset["overflowRight"] = String(
        elem.scrollLeft + elem.clientWidth < elem.scrollWidth,
      );
    };
    listener();
    elem.addEventListener("scroll", listener);
    return () => elem.removeEventListener("scroll", listener);
  }, []);
  return (
    <TableOuter>
      <TableScroll ref={outer}>
        <TableInner {...props} />
      </TableScroll>
    </TableOuter>
  );
}
