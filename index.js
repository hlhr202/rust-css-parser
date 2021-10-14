import { lex } from "./pkg/rust_css_parser_test.js";

const source = `@variable: #999;

@variable2: #fff;

@import url();

@media only screen and (max-width: 1000px) {
    color: white;
    font-size: 10px;
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
}`;

console.log(lex(source));
