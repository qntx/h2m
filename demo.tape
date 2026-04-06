Output demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600

Hide
Type "clear"
Enter
Show

# 1. Convert a URL directly
Type "h2m https://example.com"
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 2. Extract only article content with CSS selector
Type "h2m --selector p https://example.com"
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 3. GFM mode with referenced links
Type 'h2m --gfm --link-style referenced https://example.com'
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 4. Convert local HTML with all formatting options
Type 'echo '"'"'<h1>Hello</h1><p>A <strong>bold</strong> and <em>italic</em> world.</p><ul><li>one</li><li>two</li></ul>'"'"' | h2m --gfm'
Sleep 500ms
Enter
Sleep 3s

Ctrl+L

# 5. Save output to file
Type "h2m https://example.com -o example.md && cat example.md"
Sleep 500ms
Enter
Sleep 4s
