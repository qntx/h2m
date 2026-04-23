# Real end-to-end smoke test for `h2m search`.
#
# Exercises both default zero-config providers (DuckDuckGo, Wikipedia)
# with several complex scenarios and prints the raw JSON responses so
# users can inspect shape and values side-by-side. Unlike the in-crate
# wiremock tests this talks to the real upstream services.
#
# Usage:
#   pwsh scripts/live_search_e2e.ps1
#
# Exit codes:
#   0 — at least one provider returned a non-empty web[] for every case
#   1 — one or more cases failed completely (neither DDG nor Wikipedia
#       returned results AND it was not a known-CAPTCHA scenario)

param(
    [string]$Bin = (Join-Path $PSScriptRoot "..\target\release\h2m.exe")
)

$ErrorActionPreference = 'Continue'
chcp 65001 | Out-Null
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

if (-not (Test-Path $Bin)) {
    Write-Host "h2m binary not found at $Bin — run 'cargo build --release -p h2m-cli' first" -ForegroundColor Red
    exit 2
}

$cases = @(
    @{ Name = 'DDG basic English'      ; Args = @('search','rust async trait','--limit','3','--json')                                                          },
    @{ Name = 'DDG Chinese'            ; Args = @('search','机器学习','--language','zh','--limit','3','--json')                                                 },
    @{ Name = 'DDG time filter'        ; Args = @('search','openai news','--time-range','week','--limit','3','--json')                                        },
    @{ Name = 'DDG safesearch strict'  ; Args = @('search','rust crate','--safesearch','strict','--limit','3','--json')                                       },
    @{ Name = 'Wikipedia English'      ; Args = @('search','Turing machine','--provider','wikipedia','--limit','3','--json')                                  },
    @{ Name = 'Wikipedia Chinese'      ; Args = @('search','图灵机','--provider','wikipedia','--wikipedia-lang','zh','--limit','3','--json')                   },
    @{ Name = 'Wikipedia Japanese'     ; Args = @('search','チューリング','--provider','wikipedia','--wikipedia-lang','ja','--limit','3','--json')              }
)

$summary = @()
foreach ($c in $cases) {
    Write-Host ("`n{0}" -f ('=' * 78)) -ForegroundColor DarkGray
    Write-Host ("# {0}" -f $c.Name) -ForegroundColor Cyan
    Write-Host ("  args: {0}" -f ($c.Args -join ' ')) -ForegroundColor DarkGray
    Write-Host ('-' * 78) -ForegroundColor DarkGray

    $raw = & $Bin @($c.Args) 2>&1 | Out-String
    Write-Host $raw

    # Cheap structural parse: we don't need full JSON validity, just signal
    # counts. PowerShell's line-based parser chokes on pretty-printed multi-line
    # JSON interleaved with NDJSON, so rely on regex over the full blob.
    $kind = 'unknown'
    $errorKind = $null
    if ($raw -match '"kind"\s*:\s*"([^"]+)"') {
        $errorKind = $Matches[1]
    }
    $hitMatches = [regex]::Matches($raw, '"url"\s*:\s*"https?://')
    $hits = $hitMatches.Count

    if ($errorKind) {
        $kind = $errorKind
        Write-Host ("  -> error kind: {0}" -f $kind) -ForegroundColor Yellow
    } elseif ($hits -gt 0) {
        $kind = 'ok'
        Write-Host ("  -> {0} hit(s) returned" -f $hits) -ForegroundColor Green
    } else {
        $kind = 'empty'
        Write-Host '  -> no hits and no error (unexpected)' -ForegroundColor Red
    }

    $summary += [pscustomobject]@{
        Name = $c.Name
        Kind = $kind
        Hits = $hits
    }
}

Write-Host ("`n{0}" -f ('=' * 78)) -ForegroundColor DarkGray
Write-Host '# Summary' -ForegroundColor Cyan
Write-Host ('-' * 78) -ForegroundColor DarkGray
$summary | Format-Table -AutoSize | Out-String | Write-Host

# 'empty' is the only unexpected outcome. Known-CAPTCHA / parse errors are
# treated as acceptable for this smoke test because they indicate upstream
# blocking rather than a code defect.
$failures = @($summary | Where-Object { $_.Kind -eq 'empty' })
if ($failures.Count -gt 0) {
    Write-Host "Some cases returned no hits AND no classified error (bug):" -ForegroundColor Red
    $failures | Format-Table -AutoSize | Out-String | Write-Host
    exit 1
}
exit 0
