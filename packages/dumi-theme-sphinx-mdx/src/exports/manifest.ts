import { manifests } from '@@/plugin-sphinx-theme/manifest';
import { useLocation } from 'dumi';

export function useNearestManifest() {
  const { pathname } = useLocation();

  const candidates = Object.keys(manifests).filter((path) => pathname.startsWith(path));
  candidates.sort((a, b) => b.length - a.length);

  if (!candidates.length) {
    return undefined;
  }

  const index = candidates[0];
  const manifest = manifests[index];

  return { index, manifest };
}
