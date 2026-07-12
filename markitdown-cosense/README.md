# markitdown-cosense

A [MarkItDown](https://github.com/microsoft/markitdown) plugin that converts Cosense notation into Markdown. Conversion runs in a Rust core (`cosense-core`) shared with a browser wasm module.

## Features
- Headings: `[* Heading]` → `# Heading`
- Text styles: `[/ italic]`, `[[bold]]`, `[*/ strong italic]`, `[*- bold strike]`, `[/- italic strike]`, `[- strikethrough]`
- Lists: indented bullets using spaces, tabs, or full-width spaces
- Code blocks: `code:language` (indented body) and fenced ``` ``` sections
- Tables: `table:Title` directives with tab-separated cells
- Links & media: `[Label https://example]`, `[img https://.../image.png]`, `[YouTube …]`, `[Twitter …]`
- Math: `code:tex` blocks → `$$ … $$`
- Tags & page links: `[tag]` → `#tag`

## Installation
```bash
pip install markitdown markitdown-cosense
```

## CLI
```bash
markitdown --use-plugins note.txt > note.md
```

## Python
```python
from markitdown import MarkItDown, StreamInfo
from markitdown_cosense import register_converters

md = MarkItDown()
register_converters(md)

with open("note.txt", "rb") as fh:
    result = md.convert_stream(fh, stream_info=StreamInfo(extension=".txt"))

print(result.text_content)
```

## License
MIT © kazu728
