Output demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 720

Hide
Type "clear"
Enter
Show

# 1. Convert a URL directly
Type "h2m convert https://example.com"
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 2. Extract article content with readable mode
Type "h2m convert -r https://blog.example.com/post"
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 3. GFM mode with JSON output
Type "h2m convert --gfm --json https://example.com"
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 4. Convert local HTML from stdin
Type 'echo '"'"'<h1>Hello</h1><p>A <strong>bold</strong> <em>world</em>.</p><ul><li>one</li><li>two</li></ul>'"'"' | h2m convert --gfm'
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 5. Web search (SearXNG) — pure search
Type 'export H2M_SEARXNG_URL=https://searx.be && h2m search "rust async trait" --limit 3'
Sleep 500ms
Enter
Sleep 4s

Ctrl+L

# 6. Search + scrape in one call (NDJSON streaming)
Type 'h2m search "rust async" --limit 2 --scrape --readable --gfm'
Sleep 500ms
Enter
Sleep 6s
