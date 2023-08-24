import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import { styled } from '@linaria/react';
import Heading from '@theme/Heading';
import Layout from '@theme/Layout';

const Header = styled.header`
  position: relative;
  overflow: hidden;
  padding: 2rem calc(var(--ifm-navbar-padding-horizontal) + 2.4rem);

  @media screen and (max-width: 996px) {
    padding: 2rem;
  }
`;

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Header>
      <Heading as="h1" className="hero__title">
        {siteConfig.title}
      </Heading>
      <p>
        Documentation toolchain for{' '}
        <a target="_blank" rel="noopener" href="https://www.secretflow.org.cn">
          SecretFlow
        </a>
        .
      </p>
      <hr></hr>
    </Header>
  );
}

export default function Home(): JSX.Element {
  return (
    <Layout title="Home">
      <HomepageHeader />
    </Layout>
  );
}
