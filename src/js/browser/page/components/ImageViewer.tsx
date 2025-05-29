import { Image } from "antd";
import type { ComponentProps, PropsWithChildren } from "react";
import { styled } from "styled-components";

const Image2 = styled(Image)<{ width?: string }>`
  width: ${(props) => props.width}px !important;
  cursor: pointer;
`;

export function ImageViewer(props: PropsWithChildren<ComponentProps<"img">>) {
  const { width, src } = props;
  if (!src) {
    return null;
  }
  return (
    <Image2
      key={src}
      src={src}
      width={width ? `${width}` : "auto"}
      preview={{ mask: false, src }}
    />
  );
}
