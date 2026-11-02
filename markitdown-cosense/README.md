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

## Which files it converts
Files with a `.txt` extension (or a stream with no extension) whose first 4 KiB contain Cosense notation. Other formats such as `.md` or `.py` are left to MarkItDown's own converters. Input is read as UTF-8.

## CLI
```bash
markitdown --use-plugins note.txt > note.md
```

When reading from stdin, pass the extension hint:
```bash
markitdown --use-plugins -x txt < note.txt > note.md
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
