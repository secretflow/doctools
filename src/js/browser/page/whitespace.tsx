import type { ReactNode } from "react";
import { cloneElement, Fragment, isValidElement } from "react";

const PUNCTUATION = "!@#$%^&([{<:'\"`/\\|.+=";

const RE_WBR = new RegExp(`([${PUNCTUATION}][^${PUNCTUATION}]*)`);

export function wordBreak(children: ReactNode): ReactNode {
  return Array.from(wordBreakIterator(children)) //
    .map((item, i) => <Fragment key={i}>{item}</Fragment>);
}

function* wordBreakIterator(children: ReactNode): Generator<ReactNode> {
  if (typeof children === "string") {
    yield* intersperse(<wbr />, children.split(RE_WBR));
  } else if (Array.isArray(children)) {
    for (const child of children) {
      yield* wordBreakIterator(child);
    }
  } else if (isValidElement(children) && children.type === "span") {
    const grandchildren = children.props["children"];
    yield cloneElement(children, undefined, wordBreak(grandchildren));
  } else {
    yield children;
  }
}

function* intersperse(k: ReactNode, a: ReactNode[]): Generator<ReactNode> {
  const iter = a[Symbol.iterator]();
  let next: ReactNode = undefined;
  while ((next = iter.next().value) !== undefined) {
    if (next) {
      yield next;
      break;
    }
  }
  while ((next = iter.next().value) !== undefined) {
    if (next) {
      yield k;
      yield next;
    }
  }
}
