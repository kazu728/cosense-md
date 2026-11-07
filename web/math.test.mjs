import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";
import { mathBlocks } from "./math.js";

const index = await readFile(new URL("./index.html", import.meta.url), "utf8");
const [, url, integrity] = index.match(/src="([^"]*\/markdown-it@[^\"]+)"\s+integrity="([^"]+)"/);
const response = await fetch(url);
assert.ok(response.ok);
const code = await response.text();
assert.equal(`sha384-${createHash("sha384").update(code).digest("base64")}`, integrity);
const context = vm.createContext({});
vm.runInContext(code, context);
const md = context.markdownit().use(mathBlocks);

test("math keeps blank lines and embedded dollar signs verbatim", () => {
  const source = "$$\n\n\\text{if $x>0$}\n\n$$";
  assert.equal(md.render(source), "<div>$$\n\n\\text{if $x&gt;0$}\n\n$$</div>\n");
});

test("dollar text and following parenthesized prose stay literal", () => {
  assert.equal(md.render("a$b$c echo $A$B $PATH$"), "<p>a$b$c echo $A$B $PATH$</p>\n");
  assert.ok(md.render("$$\nx^2\n$$\n(1)式より明らか").endsWith("<p>(1)式より明らか</p>\n"));
});

test("nested and adjacent math blocks keep their list parent", () => {
  const html = md.render("- 分散\n\n  $$\n  x^2\n  $$\n\n  $$\n  y^2\n  $$");
  assert.equal((html.match(/<div>/g) ?? []).length, 2);
  assert.ok(html.startsWith("<ul>\n<li>"));
  assert.ok(html.endsWith("</li>\n</ul>\n"));
});

test("empty math blocks are preserved", () => {
  assert.equal(md.render("$$\n$$"), "<div>$$\n$$</div>\n");
});

test("math source is escaped before HTML sanitization", () => {
  const html = md.render("$$\n\\text{<script>alert(1)</script>}\n$$");
  assert.ok(!html.includes("<script>"));
  assert.ok(html.includes("&lt;script&gt;"));
});

test("code fences and unclosed delimiters keep their contents", () => {
  assert.ok(md.render("```tex\n$$\nx^2\n$$\n```").startsWith('<pre><code class="language-tex">'));
  const html = md.render("$$\nx^2\nparagraph");
  assert.ok(!html.includes("<div>"));
  assert.ok(html.includes("paragraph"));
});
