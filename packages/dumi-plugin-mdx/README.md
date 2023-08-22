# dumi-plugin-mdx

[![npm](https://img.shields.io/npm/v/%40mdx-js%2Fmdx?label=Powered%20by%20%40mdx-js%2Fmdx)](https://www.npmjs.com/package/@mdx-js/mdx)

Write your Dumi docs in [MDX](https://mdxjs.com/).

MDX lets you use proper JSX syntax in Markdown documents. No more
[fenced code magic](https://d.umijs.org/guide/write-demo#代码块) or [pretending it's HTML](https://d.umijs.org/theme/global-component#实践与限制).

```md
import { CoolComponent } from 'cool-component';
import { version } from 'cool-component/package.json';

# Cool react component

[![npm](https://img.shields.io/npm/v/%40mdx-js%2Fmdx?label=Powered%20by%20%40mdx-js%2Fmdx)](https://www.npmjs.com/package/@mdx-js/mdx)

> Version: {version}

Check out this cool component:

<CoolComponent coolness={10} style={{ color: "#fcb32c" }} />
```

## Who is using this?

- [隐语 SecretFlow 文档](https://www.secretflow.org.cn/docs/secretflow)

## Install and use

**This package is NOT PRODUCTION-READY.** It does not support all Dumi features, such as
component demos. Use at your own risk.

1. Install in a Dumi project.

   ```bash
   npm i -D dumi-plugin-mdx
   # or yarn, or pnpm
   ```

2. Add `dumi-plugin-mdx` to your `.dumirc.ts`.

   ```diff
   import { defineConfig } from 'dumi';

   export default defineConfig({
   + plugins: ['dumi-plugin-mdx'],
   });
   ```

3. Put .mdx files in your `docs` folder. See [MDX documentation](https://mdxjs.com/docs/what-is-mdx/) on how to write one.

   > You MUST use the `.mdx` extension. `.md` files will still be parsed as Markdown.

   ```md
   # Hello, world!

   <div className="note">
     > Some notable things in a block quote!
   </div>
   ```

4. Build away!

   ```bash
   npm run build
   # npm exec dumi build, or yarn, or pnpm
   ```

You can write .mdx files along side your existing .md files. Dumi's [routing](https://d.umijs.org/guide/conventional-routing) behavior will still apply.

```
docs
├── index.mdx
├── stable
│   └── existing-doc.md
└── next
    └── new-doc.mdx
.dumirc.ts
```
