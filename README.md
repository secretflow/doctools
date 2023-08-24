# @secretflow/doctools

[SecretFlow] 文档构建工具。

[SecretFlow]: https://secretflow.org.cn/

## 安装

```bash
pip install secretflow-doctools
```

## 使用

`secretflow_doctools` 以 Sphinx 插件的形式提供，在你的 `conf.py` 中添加插件：

```python
extensions = [
    # ...
    "secretflow_doctools",
]
```

Sphinx 命令：

```bash
python -m sphinx -b mdx source build/mdx
```

其中：

- `-b mdx` —— 生成 [MDX] (Markdown + JSX) 文件，即本插件的主要功能
- `source` —— 文档源文件根目录（包含 `conf.py` 文件的目录）
- `build/mdx` —— 输出目录，可以是任意目录

[MDX]: https://mdxjs.com/

其余 CLI 选项和 `sphinx-build` 一致

或者使用 Makefile：

```bash
make mdx
```
