import fs from "fs";
import { parse } from "./pkg/rust_css_parser_test.js";

fs.promises
    .readFile("./test/test.less", { encoding: "utf-8" })
    .then((source) => {
        console.log(parse(source));
    });
