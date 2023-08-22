import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import { styled } from '@linaria/react';
import Heading from '@theme/Heading';
import Layout from '@theme/Layout';

const Header = styled.header`
  position: relative;
  overflow: hidden;
  padding: 4rem 0;
  text-align: center;

  @media screen and (max-width: 996px) {
    padding: 2rem;
  }
`;

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Header>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
      </div>
    </Header>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Hello from ${siteConfig.title}`}
      description="Description will go into a meta tag in <head />"
    >
      <HomepageHeader />
    </Layout>
  );
}
