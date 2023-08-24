import { useOutlet } from 'dumi';
import { createGlobalStyle } from 'styled-components';

const GlobalStyle = createGlobalStyle`
  html {
    margin: 0;
  }
`;

const GlobalLayout = () => {
  const outlet = useOutlet();
  return (
    <>
      <GlobalStyle />
      {outlet}
    </>
  );
};

export default GlobalLayout;
