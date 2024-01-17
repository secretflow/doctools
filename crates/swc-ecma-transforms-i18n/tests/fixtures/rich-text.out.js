jsx("p", {
    children: jsx(Trans, {
        id: "******",
        message:
            "The quick <0>brown</0> fox <1>jumps</1> over the <2>lazy <3>dog</3></2>.",
        components: {
            0: jsx("em", {}),
            1: jsx("strong", {}),
            2: jsx("a", {
                href: gettext({
                    id: "******",
                    message:
                        "https://en.wikipedia.org/wiki/The_quick_brown_fox_jumps_over_the_lazy_dog",
                }),
            }),
            3: jsx("span", {}),
        },
    }),
});
