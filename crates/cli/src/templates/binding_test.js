import { test, expect } from "bun:test";
import Parser from "tree-sitter";

test("can load grammar", async () => {
  const parser = new Parser();
  const { default: language } = await import("./index.js");
  expect(() => parser.setLanguage(language)).not.toThrow();
});
