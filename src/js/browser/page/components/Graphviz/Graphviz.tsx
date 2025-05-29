import { Graphviz } from "graphviz-react";
import { styled } from "styled-components";

const Container = styled.div`
  > div {
    max-width: 95%;
    margin: 1rem auto;
  }
`;

const GraphvizWrapper = ({ code }: { code: string }) => (
  <Container>
    <Graphviz
      dot={code}
      options={{ width: "100%", height: "100%", fit: 1, scale: 1 }}
    />
  </Container>
);

GraphvizWrapper.displayName = "Graphviz";

export { GraphvizWrapper as Graphviz };
