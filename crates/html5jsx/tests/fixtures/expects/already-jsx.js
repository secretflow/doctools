jsxs(Fragment, {
    "children": [
        jsxs("app", {
            "children": [
                "\n    ",
                jsxs("div", {
                    "children": [
                        "\n        ",
                        jsx("blockquote", {
                            children: "A quick brown {fox} jumps over the lazy dog."
                        }),
                        "\n        ",
                        jsx("a", {
                            "href": '{"https://example.com"}',
                            children: "This is a link"
                        }),
                        '\n        {"This is a text node"}\n        ',
                        jsx("h1", {
                            children: "Hello, world!"
                        }),
                        "\n    "
                    ]
                }),
                "\n"
            ]
        }),
        "\n"
    ]
});
