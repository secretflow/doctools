# secretflow-docpack

## General workflow

- Every `sphinx-build` results in a [**standard**][npm-package-json] [NPM package](#npm-package-structure);
- Every build is **versioned** and **published** to NPM;
- Docs are consumed however NPM packages are consumed — `npm install`, or through [one][jsdelivr] [of][esm-sh] [the][unpkg] [many][cdnjs] CDN providers.

[npm-package-json]: https://docs.npmjs.com/cli/v10/configuring-npm/package-json
[jsdelivr]: https://jsdelivr.com/
[esm-sh]: https://esm.sh/
[unpkg]: https://unpkg.com/
[cdnjs]: https://cdnjs.com/

## NPM package structure

- `api` - see [API extraction](#api-extraction)
- `assets/` - see [static asset bundling](#static-asset-bundling)
  - `static/<locale>/**/*` - static files
  - `client.js` - ES module: asset names + locales to URLs
  - `server.js` - ES module: asset names + locales to static imports
- `indices/<locale>.js` - ES modules: full-text search index
- `package.json` - NPM package metadata; see [versioning](#versioning)
- `pages/<locale>/**/*.js` - ES modules: pages as React components
- `redirects.js` - ES module: redirects (page move), see [Redirects](#redirects)
- `sitemap.js` - ES module: metadata: paths, fragments, titles, and locale availability; all languages

## Versioning

Every build has a unique version identifier, consisting of the following parts:

- _Semver-like_ version: a [semver] or [PEP-440] version number, e.g. `1.2.3`, `1.2.3a4`, or `1.2.3-rc4`;
  - This can be sourced from git tags or provided as Sphinx options;
  - NPM requires strict semantic versions, PEP-440 versions will be converted to an equivalent semver string with as little loss of information as possible; see the [`semver` package][node-semver];
- _Build number_: MUST come from `git rev-parse --short HEAD`;
  - This will be appended to the version string as `build.<commit>`, e.g. `build.0292fd8`;
  - Connecting character depends on whether additional segments are already present in the version string, see examples below; note that this is never `+` because `+build` will be omitted by NPM;

Builds SHOULD have a _semver-like_ version. If it is unavailable at build time, `0.0.0` will be used.

Builds MUST have a _build number_. If it is unavailable at build time, (e.g. if not building from a git repository), `version` will be omitted from `package.json` entirely, and the resulting package will not be publishable.

Additionally, Git tags SHOULD become NPM [dist tags][dist-tags].

Examples:

- commit `0292fd8`, nearest git tag `1.2.3` → `1.2.3-build.0292fd8`
- commit `0292fd8`, `__version__ = (2, 1, 0, "rc", 3)` → `2.1.0-rc.3.build.0292fd8`
- commit `0292fd8`, no version provided → `0.0.0-build.0292fd8`

[semver]: https://semver.org/
[PEP-440]: https://www.python.org/dev/peps/pep-0440/
[node-semver]: https://www.npmjs.com/package/semver
[dist-tags]: https://docs.npmjs.com/cli/v10/commands/npm-dist-tag

TODO: formal description of version extraction algorithm

## API extraction

In general, API documentation are extracted as _structured data_ from _source code_ and then added to resulting NPM package.

We are primarily interested in extracting the following types of information:

- **Data types**: structs, Protocol Buffers messages, Pydantic models, etc.
- **Functions**: Python functions, but also RPC methods and HTTP handlers.

We aim to represent such information in formats that are _machine-readable,_ such as JSON Schema and Protocol Buffers.

The following are high-level descriptions of input and output formats we plan to support.

TODO: formal description API data formats

### Protocol Buffers & Family

This encompasses all data types and services defined through `.proto` files.

The idea is to generate **machine-readable and language-agnostic** representations of Protobuf messages and services using `protoc` — thus structured docs are generated in tandem with Go/Python/Java/... stub files.

#### Output format

Output will most likely be **JSON Schema** and closely related specs (Swagger/OpenAPI).

No matter which plugins/tools we choose (below), it is likely we will need to annotate the resulting schema with additional metadata in order to support searching/cross-referencing on the website.

#### `protoc` plugin candidates

- [pseudomuto/protoc-gen-doc] — _already used by SPU and SCQL_ to generate Markdown docs (messages are rendered as tables). We could instead utilize its JSON output.
- [chrusty/protoc-gen-jsonschema] — generates JSON Schema from `.proto` files. A slight problem is that JSON Schema may not expressive all of Protobuf's features (e.g. integers of different sizes); additionally, JSON Schema has no concept of services or API endpoints.
- **[grpc-ecosystem/grpc-gateway]** — generates HTTP server stubs from gRPC definitions, and has a built-in `protoc-gen-openapiv2` for generating Swagger specs. Requires annotating `.proto` files with HTTP-specific metadata.
- **[google/gnostic]** — generates OpenAPI v3 specs from gRPC definitions. Also requires annotating `.proto` files with HTTP-specific metadata.

[pseudomuto/protoc-gen-doc]: https://github.com/pseudomuto/protoc-gen-doc
[chrusty/protoc-gen-jsonschema]: https://github.com/chrusty/protoc-gen-jsonschema
[grpc-ecosystem/grpc-gateway]: https://github.com/grpc-ecosystem/grpc-gateway
[google/gnostic]: https://github.com/google/gnostic

#### Recommendation

**We recommend [Buf][buf-build] as our tooling of choice for all things Protocol Buffers related.**

**For a demo of incorporating Buf into `secretflow/scql`, see <https://github.com/tonywu6/scql/blob/main/buf-demo.md>.**

[buf-build]: https://buf.build/

### Pydantic

### Python symbols

## Static asset bundling

## Redirects

## i18n

- Docusaurus: Translations are entirely separate instances
  - There are no defined workflows for maintaining consistency across languages
- Format.JS: Canonical message IDs, messages in separate files
  - Does not work well for prose content, which is the majority of documentation
- Sphinx: Prose _are_ message IDs, `gettext` during build

Research: [lingui] which has SWC plugins and supports "React components inside localized messages"

[lingui]: https://lingui.dev/

## Statuspage
