import { useRef } from 'react';

export function useUntrackedValue<T>(value: T) {
  const ref = useRef(value);
  ref.current = value;
  return ref;
}
