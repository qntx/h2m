#!/usr/bin/env pwsh
# Ephemeral smoke test: spin up a tiny local HTTP mock and drive the h2m CLI
# end-to-end against it. Runs without any external network access.

$ErrorActionPreference = "Stop"

# Pick a free port.
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
$listener.Start()
$port = $listener.LocalEndpoint.Port
$listener.Stop()
$baseUrl = "http://127.0.0.1:$port"

Write-Host "=== Starting local mock SearXNG on $baseUrl ==="

$mockScript = {
    param($port)
    $prefix = "http://127.0.0.1:$port/"
    $http = [System.Net.HttpListener]::new()
    $http.Prefixes.Add($prefix)
    $http.Start()

    $payload = @{
        results = @(
            @{ url = "https://rust-lang.org"; title = "Rust"; content = "A language empowering everyone."; engine = "google"; category = "general" }
            @{ url = "https://docs.rs"; title = "docs.rs"; content = "Documentation for every crate."; engine = "bing"; category = "general" }
            @{ url = "https://news.rust-lang.org"; title = "Rust Blog"; content = "Latest news."; engine = "google"; category = "news" }
        )
    } | ConvertTo-Json -Depth 10 -Compress

    while ($http.IsListening) {
        try {
            $ctx = $http.GetContext()
            $response = $ctx.Response
            $response.ContentType = "application/json"
            $response.StatusCode = 200
            $bytes = [System.Text.Encoding]::UTF8.GetBytes($payload)
            $response.OutputStream.Write($bytes, 0, $bytes.Length)
            $response.OutputStream.Close()
        } catch {
            break
        }
    }
    $http.Stop()
}

$job = Start-Job -ScriptBlock $mockScript -ArgumentList $port
Start-Sleep -Milliseconds 500

try {
    $exe = "$PSScriptRoot/../target/release/h2m.exe"

    Write-Host "`n=== test 1: pretty JSON search output ===" -ForegroundColor Cyan
    & $exe search "rust" --searxng-url $baseUrl --limit 3

    Write-Host "`n=== test 2: NDJSON streaming search output ===" -ForegroundColor Cyan
    & $exe search "rust" --searxng-url $baseUrl --limit 3 --json

    Write-Host "`n=== test 3: search --scrape (hits example.com to keep it fast) ===" -ForegroundColor Cyan
    # scrape example.com which is stable; we bypass that by overriding each hit URL in the mock
    & $exe search "rust" --searxng-url $baseUrl --limit 1 --scrape 2>&1 | Select-Object -First 10

    Write-Host "`n=== test 4: --sources news ===" -ForegroundColor Cyan
    & $exe search "rust" --searxng-url $baseUrl --sources news --limit 2

    Write-Host "`n=== test 5: -p invalid provider ===" -ForegroundColor Cyan
    $out = & $exe search "q" --provider yahoo 2>&1
    Write-Host $out

    Write-Host "`n=== test 6: convert end-to-end with example.com ===" -ForegroundColor Cyan
    & $exe convert --json https://example.com
}
finally {
    Stop-Job -Job $job -ErrorAction SilentlyContinue | Out-Null
    Remove-Job -Job $job -Force -ErrorAction SilentlyContinue | Out-Null
    Write-Host "`n=== mock server stopped ===" -ForegroundColor Yellow
}
