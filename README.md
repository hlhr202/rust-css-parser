# CSS(less like) parser written in rust (WIP)

This project aims to implement a CSS(less like) parser in rust. Currently the code is targeting the PostCSS AST.
## Features:
### Nestable CSS Standard
- [x] rule
- [x] prop
- [x] declaration
  - [x] url
  - [x] hex
  - [x] string
  - [x] number
  - [x] calc
  - [x] var
  - [x] !important
- [x] at rule
  - [x] media query
  - [x] import
  - [x] keyframe
  - [x] variable
- [ ] selector
  - [x] block
  - [x] class
  - [x] id
  - [x] data attribute 
  - [x] &
  - [x] ,
  - [x] +
  - [x] *
  - [x] sudo class
  - [ ] more W3C standard to be tested...
- [ ] comment
- [ ] function?

### Diagnostic
- [ ] location
- [ ] error report

### APIs
- [ ] transformer
- [ ] generator
- [x] wasm

### Example
- input

```less
@variable: #999;

@variable2: #fff;

@variable3: white !important;

@import url();

@media only screen and (max-width: 1000px) {
    color: white;
    font-size: 10px !important;
}

main {
    * {
        font-size: 3em;
    }
    color: #999;
    -webkit-line-clamp: 3;
    h3 {
        font-size: large;
        background: url("https://www.baidu.com");
    }
    div + p {
        list-style: "|";
    }
    .test-1, #test2 {
        --theme-color: var(--white);
    }
    [data-attr] {
        background-color: white;
        left: calc(100% - 10px);
    }
    &.img {
        width: fit-content;
    }
    & {
        div {
            overflow: initial;
        }
    }
    #what {
        right: 0;
    }
}

.test-class1 {
    color: white;
}

@keyframes anim {
    from {
        left: 0%;
    }
    to {
        right: 100%;
    }
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
        nodes: None,
    },
    Atrule {
        type: "atrule",
        name: "variable2",
        params: "#fff",
        value: Some(
            "#fff",
        ),
        nodes: None,
    },
    Atrule {
        type: "atrule",
        name: "variable3",
        params: "white !important",
        value: Some(
            "white !important",
        ),
        nodes: None,
    },
    Atrule {
        type: "atrule",
        name: "import",
        params: "url()",
        value: None,
        nodes: None,
    },
    Atrule {
        type: "atrule",
        name: "media",
        params: "only screen and (max-width 1000px)",
        value: None,
        nodes: Some(
            [
                Decl {
                    type: "decl",
                    prop: "color",
                    value: "white",
                    important: None,
                },
                Decl {
                    type: "decl",
                    prop: "font-size",
                    value: "10px ",
                    important: Some(
                        true,
                    ),
                },
            ],
        ),
    },
    Rule {
        type: "rule",
        selector: "main",
        nodes: [
            Rule {
                type: "rule",
                selector: "*",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "font-size",
                        value: "3em",
                        important: None,
                    },
                ],
            },
            Decl {
                type: "decl",
                prop: "color",
                value: "#999",
                important: None,
            },
            Decl {
                type: "decl",
                prop: "-webkit-line-clamp",
                value: "3",
                important: None,
            },
            Rule {
                type: "rule",
                selector: "h3",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "font-size",
                        value: "large",
                        important: None,
                    },
                    Decl {
                        type: "decl",
                        prop: "background",
                        value: "url(\"https://www.baidu.com\")",
                        important: None,
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "div + p",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "list-style",
                        value: "\"|\"",
                        important: None,
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: ".test-1, #test2",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "--theme-color",
                        value: "var(--white)",
                        important: None,
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "[data-attr]",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "background-color",
                        value: "white",
                        important: None,
                    },
                    Decl {
                        type: "decl",
                        prop: "left",
                        value: "calc(100% - 10px)",
                        important: None,
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
                        important: None,
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "&",
                nodes: [
                    Rule {
                        type: "rule",
                        selector: "div",
                        nodes: [
                            Decl {
                                type: "decl",
                                prop: "overflow",
                                value: "initial",
                                important: None,
                            },
                        ],
                    },
                ],
            },
            Rule {
                type: "rule",
                selector: "#what",
                nodes: [
                    Decl {
                        type: "decl",
                        prop: "right",
                        value: "0",
                        important: None,
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
                important: None,
            },
        ],
    },
    Atrule {
        type: "atrule",
        name: "keyframes",
        params: "anim",
        value: None,
        nodes: Some(
            [
                Rule {
                    type: "rule",
                    selector: "from",
                    nodes: [
                        Decl {
                            type: "decl",
                            prop: "left",
                            value: "0%",
                            important: None,
                        },
                    ],
                },
                Rule {
                    type: "rule",
                    selector: "to",
                    nodes: [
                        Decl {
                            type: "decl",
                            prop: "right",
                            value: "100%",
                            important: None,
                        },
                    ],
                },
            ],
        ),
    },
]
```