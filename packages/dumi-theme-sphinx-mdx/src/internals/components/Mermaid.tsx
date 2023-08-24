import mermaid from 'mermaid';
import { useRef, useEffect } from 'react';
import styled from 'styled-components';

const Container = styled.div<{ align?: 'flex-start' | 'center' | 'flex-end' }>`
  display: flex;
  flex-flow: column nowrap;
  align-items: ${({ align }) => align || 'center'};
`;

mermaid.initialize({ startOnLoad: false });

// looks like mermaid isn't thread-safe
const queue: (() => Promise<void>)[] = [];

const FLEX_ALIGN = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
} as const;

export function Mermaid({
  code,
  align = 'center',
}: {
  code?: string;
  align?: 'left' | 'center' | 'right';
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current) {
      return;
    }
    const elem = ref.current;
    const render = async () => {
      await mermaid.run({ nodes: [elem] });
      // delete this task from the task queue
      queue.splice(queue.findIndex(render), 1);
      // if there are more tasks, run the next one
      queue.shift()?.();
    };
    queue.push(render);
    // if this is the only task, run it
    if (queue.length === 1) {
      render();
    }
  }, []);

  return (
    <Container ref={ref} align={FLEX_ALIGN[align] ?? 'center'} className="mermaid">
      {code}
    </Container>
  );
}
