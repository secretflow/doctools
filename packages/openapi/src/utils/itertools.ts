export const maybeCall = <T, A extends unknown[]>(
  fn: T | ((...args: A) => T),
  ...args: A
) => (fn instanceof Function ? fn(...args) : fn);

// https://stackoverflow.com/questions/37128624/terse-way-to-intersperse-element-between-all-elements-in-javascript-array#comment106526535_37129036
export const intersperse = <T>(arr: T[], sep: T | ((i: number) => T)): T[] =>
  arr.flatMap((x, i) => [maybeCall(sep, i), x]).slice(1);
