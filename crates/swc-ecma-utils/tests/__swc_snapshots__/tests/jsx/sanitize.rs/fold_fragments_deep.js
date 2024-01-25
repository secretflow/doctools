jsxs("div", {
    children: [
        "Lorem",
        jsx("span", {
            children: jsxs(Fragment, {
                children: [
                    jsx("span", {
                        children: "ipsum"
                    }),
                    jsxs("strong", {
                        children: [
                            "dolor",
                            jsx("span", {
                                children: "sit"
                            }),
                            "amet"
                        ]
                    })
                ]
            })
        })
    ]
});
