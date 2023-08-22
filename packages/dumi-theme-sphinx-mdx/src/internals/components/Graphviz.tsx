import { Graphviz } from 'graphviz-react';
import styled from 'styled-components';

const Container = styled.div`
  > div {
    display: flex;
  }
`;

const GraphvizWrapper = ({ code }: { code: string }) => {
  return (
    <Container>
      <Graphviz dot={code} />
    </Container>
  );
};

export { GraphvizWrapper as Graphviz };
