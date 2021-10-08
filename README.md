# CSS(less like) parser written in rust (WIP)

I aim to implement a CSS(less like) parser in rust. Currently the code is targeting the PostCSS AST.
### TODO:
- [x] rule
- [x] prop
- [x] declaration
- [x] selector
- [ ] at rule (half complete for variable)
- [ ] location
- [ ] error report
- [ ] less function
- [ ] transformer
- [ ] generator

### Example
- input

```less
@variable: #999;
@variable2: #fff;

main {
    color: #999;
    -webkit-line-clamp: 3;
    h3 {
        font-size: large;
        background: url("https://www.baidu.com");
    }
    [data-attr] {
        background-color: white;
        left: calc(100% - 10px);
    }
    &.img {
        width: fit-content;
    }
}

.test-class1 {
    color: white;
}
```

- output

```js
[
    Atrule {
        type: "atrule",
        name: "variable",
        params: "#999",
        value: Some(
            "#999",
        ),
    },
    Atrule {
        type: "atrule",
        name: "variable2",
        params: "#fff",
        value: Some(
            "#fff",
        ),
    },
    Rule {
        type: "rule",
        selector: "main",
        nodes: [
            Decl {
                type: "decl",
                prop: "color",
                value: "#999",
            },
            Decl {
                type: "decl",
                prop: "-webkit-line-clamp",
                value: "3",
            },
            Rule {
                type: "rule",
                selector: "h3",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "font-size",
                        value: "large",
                    },
                    Decl {
                        type: "decl",
                        prop: "background",
                        value: "url(\"https://www.baidu.com\")",
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "data-attr",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "background-color",
                        value: "white",
                    },
                    Decl {
                        type: "decl",
                        prop: "left",
                        value: "calc(100% - 10px)",
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "&.img",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "width",
                        value: "fit-content",
                    },
                ],
            },
        ],
    },
    Rule {
        type: "rule",
        selector: ".test-class1",
        nodes: [
            Decl {
                type: "decl",
                prop: "color",
                value: "white",
            },
        ],
    },
]
```