import type { ComponentProps } from "react";
import { styled } from "styled-components";

const Icon = styled.span`
  display: inline-block;
  flex: 0 0 auto;
  width: 1em;
  height: 1em;
  font-size: 1em;
  color: inherit;
  border-color: transparent;
  border-radius: 50%;

  > svg {
    display: block;
    width: 100%;
    height: 100%;
    animation-name: spin;
    animation-duration: 0.8s;
    animation-timing-function: linear;
    animation-delay: 0s;
    animation-iteration-count: infinite;
    animation-direction: normal;

    @keyframes spin {
      0% {
        transform: rotate(0);
      }

      100% {
        transform: rotate(360deg);
      }
    }
  }
`;

export function Spinner(props: ComponentProps<typeof Icon>) {
  return (
    <Icon {...props}>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        version="1.1"
        viewBox="0 0 512 512"
        fill="currentColor"
      >
        <path d="M256,512c-68.38,0-132.67-26.63-181.02-74.98C26.63,388.67,0,324.38,0,256c0-13.25,10.75-24,24-24s24,10.75,24,24c0,114.69,93.31,208,208,208,13.25,0,24,10.75,24,24s-10.75,24-24,24Z" />
      </svg>
    </Icon>
  );
}
