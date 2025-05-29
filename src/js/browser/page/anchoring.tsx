import type {
  ComponentProps,
  JSXElementConstructor,
  PropsWithChildren,
  ReactNode,
} from "react";
import {
  createContext,
  createElement,
  Fragment,
  useContext,
  useEffect,
  useRef,
} from "react";
import { useLocation, useNavigationType, NavigationType } from "react-router";
import { styled } from "styled-components";

import { theme } from "../theme";

import { wordBreak } from "./whitespace";

type Scrollable = () => HTMLElement | null;

const FragmentContext = createContext<{
  scrollable: Scrollable;
  decision: ScrollingDecision;
}>({
  scrollable: () => null,
  decision: "reset",
});

type ScrollingDecision = "restore" | "reset" | "focus";

function useScrollingDecision(): ScrollingDecision {
  const { hash } = useLocation();

  const navigationType = useNavigationType();

  const initialNavigation = useRef(true);
  switch (navigationType) {
    case NavigationType.Push:
    case NavigationType.Replace:
      initialNavigation.current = false;
  }

  switch (navigationType) {
    case NavigationType.Pop:
      if (initialNavigation.current) {
        if (hash) {
          return "focus";
        } else {
          return "reset";
        }
      } else {
        return "restore";
      }
    case NavigationType.Push:
    case NavigationType.Replace:
      if (hash) {
        return "focus";
      } else {
        return "reset";
      }
  }
}

export function ScrollRestore({
  scrollable: getScrollable,
  children,
}: PropsWithChildren<{
  scrollable: Scrollable;
}>) {
  const { pathname, hash } = useLocation();

  const decision = useScrollingDecision();

  useEffect(() => {
    const { scrollRestoration } = history;
    history.scrollRestoration = "manual";
    return () => void (history.scrollRestoration = scrollRestoration);
  }, []);

  const positions = useRef<Record<string, number>>({});

  useEffect(() => {
    const { current } = positions;
    const record = (location: Pick<Location, "pathname" | "hash">) => {
      const { pathname, hash } = location;
      current[`${pathname}${hash}`] = window.scrollY;
    };
    record(window.location);
    const listen = () => record(window.location);
    window.addEventListener("scroll", listen);
    return () => window.removeEventListener("scroll", listen);
  }, []);

  useEffect(() => {
    switch (decision) {
      case "restore": {
        const recorded = positions.current[`${pathname}${hash}`] ?? 0;
        window.scrollTo({ behavior: "instant", top: recorded });
        break;
      }
      case "reset":
        window.scrollTo({ behavior: "instant", top: 0 });
        break;
      case "focus": {
        const elem = decodeIds({ id: hash.slice(1) })
          .map((id) => document.getElementById(id))
          .filter((x) => x)
          .shift();
        if (elem) {
          if (elem.clientWidth) {
            elem.scrollIntoView({ behavior: "instant" });
          } else {
            // work around main content not having a box
            // when sidebar is showing on mobile
            const observer = new ResizeObserver(() => {
              if (!elem.clientWidth) {
                return;
              }
              elem.scrollIntoView({ behavior: "smooth" });
              observer.disconnect();
            });
            observer.observe(elem);
          }
        }
        break;
      }
    }
  }, [decision, hash, pathname]);

  const scrollable = useRef(getScrollable);
  scrollable.current = getScrollable;

  return (
    <FragmentContext.Provider
      value={{
        scrollable: scrollable.current,
        decision,
      }}
    >
      {children}
    </FragmentContext.Provider>
  );
}

export function useScrollParent() {
  return useContext(FragmentContext).scrollable;
}

type SupportedElement =
  | keyof JSX.IntrinsicElements
  | (JSXElementConstructor<PropsWithChildren<{ id?: string; className?: string }>> & {
      displayName?: string;
    });

export function highlighted<T extends SupportedElement>(elem: T) {
  function Styled({
    focused,
    className = "",
    children,
  }: {
    focused: boolean;
    className?: string;
    children: (className: string) => ReactNode;
  }) {
    if (focused) {
      return children(className);
    } else {
      return children("");
    }
  }

  const Highlighted = styled(Styled)`
    outline: 3px solid ${theme.colors.bg.highlight};
    background-color: ${theme.colors.bg.highlight};
  `;

  function Element({ id, className = "", ...props }: ComponentProps<T>) {
    return (
      <Highlighted focused={useFragmentFocus(id)}>
        {(style) =>
          createElement(elem, { id, className: `${className} ${style}`, ...props })
        }
      </Highlighted>
    );
  }

  if (typeof elem === "function") {
    if (elem.displayName) {
      Element.displayName = `highlighted(${elem.displayName})`;
    } else {
      Element.displayName = `highlighted(${elem.name})`;
    }
  } else {
    Element.displayName = `highlighted(${elem})`;
  }

  return Element;
}

const PermalinkButton = styled.a`
  margin-inline: 0.5rem;
  color: ${theme.colors.fg.link};
  transition: opacity 0.2s ease-in-out;

  &::before {
    font-weight: 600;
    user-select: none;
    content: "#";
  }
`;

export function permalink<T extends SupportedElement>(elem: T) {
  const wrapped = styled<T>(elem)`
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

  function Element({ id, children, ...props }: ComponentProps<T>) {
    if (!id) {
      return createElement(wrapped, props as ComponentProps<typeof wrapped>, children);
    } else {
      const href = `#${id}`;
      const label = typeof children === "string" ? children : undefined;
      return createElement(
        wrapped,
        {
          ["id"]: id,
          ["aria-label"]: label,
          ...props,
        } as ComponentProps<typeof wrapped>,
        <Fragment>
          <span>{wordBreak(children)}</span>
          <PermalinkButton
            href={href}
            aria-label={
              label ? `Direct link to ${label}` : "Direct link to this section"
            }
          >
            {
              "\u200b" /* zero-width space, allows `#` to wrap with last word in title */
            }
          </PermalinkButton>
        </Fragment>,
      );
    }
  }

  if (typeof elem === "function") {
    if (elem.displayName) {
      Element.displayName = `permalink(${elem.displayName})`;
    } else {
      Element.displayName = `permalink(${elem.name})`;
    }
  } else {
    Element.displayName = `permalink(${elem})`;
  }

  return Element;
}

function useFragmentFocus(id: string | undefined) {
  const { hash } = useLocation();
  return Boolean(id && decodeIds({ id: hash.slice(1) }).some((x) => id === x));
}

function decodeIds({ id }: { id: string }) {
  let decodedId: string;
  try {
    decodedId = decodeURIComponent(id);
  } catch {
    decodedId = id;
  }
  return [docutilsSlugger(decodedId), decodedId, id];
}

function docutilsSlugger(s: string) {
  return s
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, "-")
    .replaceAll(/^[-0-9]+|-+$/g, "");
}
