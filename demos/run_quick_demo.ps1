# RUGID RDF Demo Suite Runner (Cross-Platform)
# Quick demo sequence (1 minute)

param([switch]$Help)

if ($Help) {
    Write-Host "RUGID RDF Quick Demo Runner"
    Write-Host "Usage: .\run_quick_demo.ps1"
    Write-Host ""
    Write-Host "Runs a sequence of RDF demos for quick verification."
    exit 0
}

Write-Host "=== RUGID RDF Quick Demo (1 minute) ==="
Write-Host ""

$demos = @(
    @{name="01_hello_world.rdf"; duration=5},
    @{name="07_rotating_cube.rdf"; duration=12},
    @{name="05_platonic_solids.rdf"; duration=8}
)

foreach ($demo in $demos) {
    Write-Host "▶ Running: $($demo.name) ($($demo.duration)s)"
    $proc = Start-Process -FilePath "cargo" -ArgumentList "run", "--quiet", "--bin", "rdf_render", "--", "demos/rdf/$($demo.name)" -PassThru -NoNewWindow
    Start-Sleep -Seconds $demo.duration
    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -ErrorAction SilentlyContinue
    }
    Write-Host "✓ Complete"
    Write-Host ""
}

Write-Host "=== Quick Demo Complete ==="