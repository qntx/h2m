param([int]$Port = 8899)

$payload = @{
    results = @(
        @{ url = "https://rust-lang.org"; title = "Rust Programming Language"; content = "A language empowering everyone to build reliable and efficient software."; engine = "google"; category = "general" }
        @{ url = "https://doc.rust-lang.org"; title = "The Rust Reference"; content = "This book is the primary reference for the Rust programming language."; engine = "bing"; category = "general" }
        @{ url = "https://news.rust-lang.org"; title = "Rust Blog"; content = "Official blog of the Rust Programming Language."; engine = "google"; category = "news"; publishedDate = "2025-12-01" }
    )
} | ConvertTo-Json -Depth 10 -Compress

$listener = [System.Net.HttpListener]::new()
$listener.Prefixes.Add("http://127.0.0.1:$Port/")
$listener.Start()
Write-Host "mock SearXNG listening on http://127.0.0.1:$Port"

$bytes = [System.Text.Encoding]::UTF8.GetBytes($payload)
try {
    while ($listener.IsListening) {
        $ctx = $listener.GetContext()
        $ctx.Response.ContentType = "application/json"
        $ctx.Response.StatusCode = 200
        $ctx.Response.OutputStream.Write($bytes, 0, $bytes.Length)
        $ctx.Response.Close()
    }
} finally {
    $listener.Stop()
}
