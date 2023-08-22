# remark-inline-id

remark plugin to support inlining element `id`s using [remark-directive](https://github.com/remarkjs/remark-directive).

This plugin processes text directives from remark-directive and add `id` attributes to next sibling node's [`hProperties`](https://github.com/syntax-tree/mdast-util-to-hast#hproperties) data object, such that the resulting HAST node will have `id` attributes.
