import { createElement } from 'react';

import { defaultTokens } from '../theming/index.js';

import { useFragmentFocus } from './positioning.js';

type SupportedComponent =
  | keyof JSX.IntrinsicElements
  | React.FunctionComponent<{ id?: string | undefined; style?: React.CSSProperties }>;

export function useHighlightedElement<T extends SupportedComponent>(
  elem: T,
  id: string | undefined,
  { style, ...props }: React.ComponentProps<T>,
): React.ReactElement | null {
  const focused = useFragmentFocus(id);
  return createElement(elem, {
    id,
    style: {
      ...style,
      backgroundColor: focused ? defaultTokens.colors.highlight : undefined,
      outline: focused ? `3px solid ${defaultTokens.colors.highlight}` : undefined,
    },
    ...props,
  });
}

export function highlighted<T extends SupportedComponent>(elem: T) {
  function Element(props: React.ComponentProps<T>) {
    return useHighlightedElement(elem, props.id, props);
  }
  Element.displayName = `highlighted(${elem})`;
  return Element;
}
