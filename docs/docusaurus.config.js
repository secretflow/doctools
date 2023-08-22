// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const darkCodeTheme = require('prism-react-renderer/themes/dracula');
const lightCodeTheme = require('prism-react-renderer/themes/github');

/** @returns {Promise<import('@docusaurus/types').Config>} */
async function createConfig() {
  const { default: gfm } = await import('remark-gfm');

  const remarkPlugins = [[gfm, { singleTilde: false }]];

  return {
    title: 'SecretFlow 文档手册',
    tagline: 'Docs are cool',
    favicon: 'img/favicon.ico',

    // Set the production url of your site here
    url: 'https://secretflow-docs.vercel.app',
    // Set the /<baseUrl>/ pathname under which your site is served
    // For GitHub pages deployment, it is often '/<projectName>/'
    baseUrl: '/',

    // GitHub pages deployment config.
    // If you aren't using GitHub pages, you don't need these.
    organizationName: 'secretflow', // Usually your GitHub org/user name.
    projectName: 'doctools', // Usually your repo name.

    onBrokenLinks: 'throw',
    onBrokenMarkdownLinks: 'warn',

    // Even if you don't use internalization, you can use this field to set useful
    // metadata like html lang. For example, if your site is Chinese, you may want
    // to replace "en" with "zh-Hans".
    i18n: {
      defaultLocale: 'zh-Hans',
      locales: ['zh-Hans'],
    },

    presets: [
      [
        'classic',
        /** @type {import('@docusaurus/preset-classic').Options} */
        ({
          docs: {
            remarkPlugins,
            sidebarPath: require.resolve('./sidebars.js'),
          },
          theme: {
            customCss: require.resolve('./src/css/custom.css'),
          },
        }),
      ],
    ],

    plugins: [require.resolve('./plugins/linaria.cjs')],

    themeConfig:
      /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
      ({
        colorMode: {
          defaultMode: 'dark',
          disableSwitch: true,
        },
        // Replace with your project's social card
        image: 'img/docusaurus-social-card.jpg',
        navbar: {
          title: 'SecretFlow 文档手册',
          logo: {
            alt: 'Docs Logo',
            src: 'img/logo.svg',
          },
          items: [
            {
              type: 'docSidebar',
              sidebarId: 'monorepo',
              position: 'left',
              label: 'Monorepo',
            },
            {
              type: 'docSidebar',
              sidebarId: 'handbook',
              position: 'left',
              label: 'Handbook',
            },
          ],
        },
        footer: {
          style: 'light',
          copyright: `Copyright © ${new Date().getFullYear()} Ant Group, Inc. Built with Docusaurus.`,
        },
        prism: {
          theme: lightCodeTheme,
          darkTheme: darkCodeTheme,
        },
      }),
  };
}

module.exports = createConfig;
