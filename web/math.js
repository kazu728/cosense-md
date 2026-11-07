export function mathBlocks(md) {
  md.block.ruler.before("fence", "math_block", (state, start, end, silent) => {
    const line = (n) => state.src.slice(state.bMarks[n] + state.tShift[n], state.eMarks[n]);
    if (line(start).trim() !== "$$") return false;

    let close = start + 1;
    while (close < end) {
      if (!state.isEmpty(close) && state.sCount[close] < state.blkIndent) return false;
      if (line(close).trim() === "$$") break;
      close += 1;
    }
    if (close === end) return false;
    if (silent) return true;

    const token = state.push("math_block", "div", 0);
    token.block = true;
    token.content = state.getLines(start + 1, close, state.blkIndent, true);
    token.map = [start, close + 1];
    state.line = close + 1;
    return true;
  }, { alt: ["paragraph", "reference", "blockquote", "list"] });

  md.renderer.rules.math_block = (tokens, index) =>
    `<div>$$\n${md.utils.escapeHtml(tokens[index].content)}$$</div>\n`;
}
