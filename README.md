# @secretflow/doctools

SecretFlow 文档构建工具。

## 安装

```bash
pip install secretflow-doctools
```

## 使用

`secretflow_doctools` 以 Sphinx 插件的形式提供，在你的 `conf.py` 中：

```python
extensions = [
    # ...
    "secretflow_doctools",
]
```

`doctools` 提供一个新的 `mdx` 输出格式，生成 [MDX] (Markdown + MDX) 文件。

[MDX]: https://mdxjs.com/

使用 Sphinx 选项 `-b mdx` 来启用这个格式：

```bash
python -m sphinx -b mdx source build/mdx
```

其余 CLI 选项和 `sphinx-build` 一致

或者使用 Makefile：

```bash
make mdx
```
