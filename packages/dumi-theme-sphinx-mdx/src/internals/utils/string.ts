export function randstring(len = 16) {
  return Math.random()
    .toString(36)
    .slice(2, len + 2);
}
