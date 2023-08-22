import { useLocation, history } from 'dumi';
import { createContext, useCallback, useContext, useEffect, useRef } from 'react';

export const FragmentContext = createContext<{ focus: (id: string) => void }>({
  focus: () => undefined,
});

export function useFragmentFocus(id: string | undefined) {
  const { hash } = useLocation();
  const { focus } = useContext(FragmentContext);

  let decodedHash: string;
  try {
    decodedHash = decodeURIComponent(hash);
  } catch (e) {
    decodedHash = '';
  }

  const focused = id && decodedHash === `#${id}`;

  useEffect(() => {
    if (focused) {
      focus(id);
    }
  }, [focus, focused, id]);

  return Boolean(focused);
}

export function TrackPagePosition({ children }: React.PropsWithChildren) {
  const pageOfLastScroll = useRef<string>();

  useEffect(
    () =>
      history.listen(({ location }) => {
        if (pageOfLastScroll.current === location.pathname) {
          return;
        }
        window.scrollTo(0, 0);
        pageOfLastScroll.current = location.pathname;
      }),
    [],
  );

  const focus = useCallback((id: string) => {
    const elem = document.getElementById(id);
    if (!elem) {
      return;
    }
    elem.scrollIntoView();
  }, []);

  return (
    <FragmentContext.Provider value={{ focus }}>{children}</FragmentContext.Provider>
  );
}
