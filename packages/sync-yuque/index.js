const fs = require('fs');

const SDK = require('@yuque/sdk');
const { config } = require('dotenv');
const { arrayToTree } = require('performant-array-to-tree');
const YAML = require('yaml');

const introToc = require('./assets/intro-toc.json');

require('dotenv').config();

if (!process.env.YUQUE_TOKEN_SF) {
  console.error('Missing YUQUE_TOKEN_SF in .env file. Do nothing.');
  process.exit(0);
}

async function writeFile(params) {
  const { folderPath, fileName, content, frontMatter = '' } = params;
  if (!fs.existsSync(folderPath, { recursive: true })) {
    //check if folder already exists
    fs.mkdirSync(folderPath, { recursive: true });
  }

  fs.writeFile(folderPath + fileName, frontMatter + content, (err) => {
    if (err) {
      console.error(err);
    }
  });
}

function flatTree(tree, arr = []) {
  arr.push(tree);
  if (tree.children && tree.children.length) {
    for (const item of tree.children) {
      flatTree(item, arr);
    }
  }
  return arr;
}

function getValidToc(toc, validTocTitles) {
  const treeArr = arrayToTree(toc, { id: 'uuid', parentId: 'parent_uuid' });

  const validTreeData = treeArr.filter((arr) =>
    validTocTitles.includes(arr.data.title),
  );

  const flatTocs = [];

  validTreeData.forEach((datum) => {
    const res = flatTree(datum);
    flatTocs.push(...res);
  });

  const data = flatTocs.map((toc) => {
    if (toc.data.title === '功能体验：开放平台') {
      return { ...toc, data: { ...toc.data, title: '在线体验' } };
    }
    return toc;
  });

  return data;
}

async function fetchDoc(customToc, client, route) {
  for (let i = 0; i < customToc.length; i++) {
    const tocItem = customToc[i].data;

    // 如果是链接跳转/空链接的目录，看情况处理
    const reg = new RegExp(/^(http|#)/, 'i'); // 不区分大小写

    if (tocItem.slug && !reg.test(tocItem.slug)) {
      const detail = await client.docs.get({
        namespace: 'secret-flow/admin',
        slug: tocItem.slug,
      });

      const frontMatter = YAML.stringify({
        title: detail.title,
        toc: 'content',
        dateModified: detail.content_updated_at,
      });

      writeFile({
        folderPath: `./build/docs/${route}/`,
        fileName: `${detail.slug}.html`,
        content: detail.body_html,
        frontMatter: `---\n${frontMatter}---\n`,
      });
    }
  }
}

async function fetchToc(client, folderPath, fileName, titles) {
  const { repos } = client;
  const originToc = await repos.getTOC({ namespace: 'secret-flow/admin' });

  const toc = getValidToc(originToc, titles);

  writeFile({
    folderPath,
    fileName,
    content: JSON.stringify(toc),
  });

  return toc;
}

async function fetchData(configs) {
  const client = new SDK({
    token: configs.token,
  });

  await client.users.get();

  for (let index = 0; index < config.length; index++) {
    const docConfig = configs.docs[index];

    if (docConfig.toc.data?.length > 0) {
      await fetchDoc(docConfig.toc.data, client, docConfig.route);
    } else {
      const toc = await fetchToc(
        client,
        docConfig.toc.folderPath,
        docConfig.toc.fileName,
        docConfig.toc.titles,
      );
      await fetchDoc(toc, client, docConfig.route);
    }
  }

  console.log('Yuque md: Success Fetching!');
}

fetchData({
  token: process.env.YUQUE_TOKEN_SF,
  docs: [
    {
      toc: {
        folderPath: './build/toc/',
        fileName: `quickstart-toc.json`,
        titles: ['功能体验：开放平台'],
      }, // Doc：快速体验（平台操作手册）
      route: 'quickstart',
    },
    {
      toc: {
        data: introToc,
      }, // Doc：关于隐语
      route: 'intro',
    },
    {
      // Doc：开发者贡献指南（开发者文档）
      // TODO: custom output path
      toc: {
        data: [
          {
            data: {
              uuid: 'hqqw9n',
              type: 'DOC',
              title: '开发者贡献指南',
              slug: 'hqqw9n',
              parent_uuid: '',
              depth: 1,
            },
          },
        ],
      },
      route: 'contribution',
    },
  ],
});
