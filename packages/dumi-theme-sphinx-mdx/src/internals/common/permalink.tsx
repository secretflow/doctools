import { createElement } from 'react';
import styled from 'styled-components';

type SupportedComponent =
  | keyof JSX.IntrinsicElements
  | React.FunctionComponent<
      React.PropsWithChildren<{
        id?: string | undefined;
        className?: string | undefined;
      }>
    >;

const PermalinkButton = styled.a`
  display: inline-block;
  margin-left: 0.5rem;
  color: ${({ theme }) => theme.colors.link};
  transition: opacity 0.2s ease-in-out;
`;

export function permalink<T extends SupportedComponent>(elem: T) {
  const wrapped = styled(elem)`
    ${PermalinkButton} {
      opacity: 0;
    }

    &:hover,
    &:focus {
      ${PermalinkButton} {
        opacity: 1;
      }
    }
  `;
  function Element({ id, children, ...props }: React.ComponentProps<T>) {
    if (!id) {
      return createElement(wrapped, props, children);
    }
    return createElement(
      wrapped,
      { id, ...props },
      <>
        {children}
        <PermalinkButton href={`#${id}`} title={`Direct link to ${id}`}>
          #
        </PermalinkButton>
      </>,
    );
  }
  Element.displayName = `permalink(${elem})`;
  return Element;
}
