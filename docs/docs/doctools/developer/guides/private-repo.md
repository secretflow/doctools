---
title: Private registry with Verdaccio
---

# Play-testing with a private registry using Verdaccio

This guide shows you how to publish workspace packages to a local NPM registry using Verdaccio. This is useful if you want to test your packages locally, including installation, before publishing them to NPM.

## Setting up Verdaccio

:::info
Skip this part if you already have a Verdaccio registry running.
:::

[Verdaccio](https://verdaccio.org/) is a lightweight NPM registry that you can run locally. It's easy to set up and use. Follow the [installation](https://verdaccio.org/docs/installation) guide. In a nutshell:

```bash
# Install
pnpm i -g verdaccio
# Start
verdaccio --listen 4873
```

This will install Verdaccio as a global package using PNPM, and then start the server at port 4873. Verify by visiting [http://localhost:4873](http://localhost:4873) in your browser.

```bash
# Register a user
npm adduser --registry http://localhost:4873/
# or another port if you changed it
```

This will register yourself as a user in the registry. You must be authenticated on the registry to be able to publish packages (much like NPM).

## Publishing packages

Use the [`pnpm publish`](https://pnpm.io/cli/publish) command.

:::warning
**Make sure to include `--registry http://localhost:<port>/` in the command.** Otherwise you will be publishing to a public registry (as specified in `.npmrc`).
:::

:::tip
Make sure your Git working tree is clean. PNPM will not publish if there are uncommitted changes.
:::

To publish all packages in the workspace to the registry, run:

```bash
pnpm -r publish --registry http://localhost:4873/
```

To publish some packages, use the [`--filter`](https://pnpm.io/filtering) option:

```bash
# Publish everyting under ./packages (assuming you are in the root of the workspace)
pnpm --filter "./packages/*" publish --registry http://localhost:4873/
```

## Installing from the registry

With your packages published, you can now install them in other projects.

### Testing `@secretflow/dumi-theme-sphinx-mdx`

We will illustrate this by installing `@secretflow/dumi-theme-sphinx-mdx` in a Dumi project.

Create a new Dumi site somewhere outside of our repo (let's say ~/Desktop) with the starter kit:

```bash
cd ~/Desktop
mkdir dumi-test
cd dumi-test
pnpm create dumi
```

:::tip
During setup, remember to choose `pnpm` as the NPM client. If you would like to use other package managers, modify the commands below accordingly.
:::

Modify the `.npmrc` file (create it if it doesn't exist) in the root of the project to include the following line:

```shell title=".npmrc"
registry=http://localhost:4873/
```

You will need to rebuild `node_modules` first:

```bash
pnpm install
# Choose yes
```

Now you can install `@secretflow/dumi-theme-sphinx-mdx`:

```bash
pnpm install @secretflow/dumi-theme-sphinx-mdx
```

Verify by building the site:

```bash
pnpm run build
```

The site should build successfully using our theme.
