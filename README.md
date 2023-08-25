# @secretflow/doctools

[![License: MIT](https://img.shields.io/badge/License-MIT-blue)][MIT] ![CI status](https://github.com/secretflow/doctools/actions/workflows/ci.yml/badge.svg)

Documentation toolchain for [SecretFlow] | [SecretFlow] 文档构建工具

<span id="en-US"></span>

English | [简体中文](#zh-Hans)

## What is this?

This repository contains packages for building documentation for [SecretFlow] projects. It provides a [Sphinx] extension to build [MDX] (Markdown + JSX) documents from a Sphinx project. It also has several Node packages for integrating MDX into [Dumi].

## Install

Install _doctools_ with `pip`:

```bash
pip install secretflow-doctools
```

## Use

Add _doctools_ as a Sphinx extension to your Sphinx project's `conf.py`:

```python
extensions = [
    # ...
    "secretflow_doctools",
]
```

Then specify `mdx` as the [builder][sphinx-build-builder]:

```bash
python3 -m sphinx -b mdx [...] [sourcedir] [outputdir]
```

Or use `make`:

```bash
make mdx
```

## License

[MIT] © [SecretFlow]

---

<span id="zh-Hans"></span>

[English](#en-US) | 简体中文

## 这是什么？

这个仓库包含了用于构建 [SecretFlow] 项目文档的工具。它提供了一个 [Sphinx] 扩展，用于从 Sphinx 项目构建 [MDX]（Markdown + JSX）文档。它还包含了一些 Node 包，用于将 MDX 集成到 [Dumi] 中。

## 安装

使用 `pip` 安装这个工具：

```bash
pip install secretflow-doctools
```

## 使用

将 doctools 作为 Sphinx 插件添加到你的 Sphinx 项目的 `conf.py` 中：

```python
extensions = [
    # ...
    "secretflow_doctools",
]
```

然后将 `mdx` 指定为 [builder][sphinx-build-builder]：

```bash
python3 -m sphinx -b mdx [...] [sourcedir] [outputdir]
```

或者使用 `make`：

```bash
make mdx
```

## 许可证

[MIT] © [SecretFlow]

[SecretFlow]: https://www.secretflow.org.cn/
[Sphinx]: https://www.sphinx-doc.org/
[MDX]: https://mdxjs.com/
[Dumi]: https://d.umijs.org/
[sphinx-build-builder]: https://www.sphinx-doc.org/en/master/man/sphinx-build.html#cmdoption-sphinx-build-b
[MIT]: ./LICENSE
