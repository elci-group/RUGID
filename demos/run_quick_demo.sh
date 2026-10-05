#!/bin/bash
# RUGID RDF Demo Suite Runner
# Quick demo sequence (1 minute)
#
# Cross-platform: Also available as run_quick_demo.ps1 (PowerShell) and run_quick_demo.bat (Windows)

echo "=== RUGID RDF Quick Demo (1 minute) ==="
echo ""

demos=(
    "01_hello_world.rdf:5"
    "07_rotating_cube.rdf:12"
    "05_platonic_solids.rdf:8"
)

for demo_spec in "${demos[@]}"; do
    IFS=':' read -r demo duration <<< "$demo_spec"
    echo "▶ Running: $demo (${duration}s)"
    cargo run --quiet --bin rdf_render -- "demos/rdf/$demo" &
    PID=$!
    sleep "$duration"
    kill $PID 2>/dev/null
    echo "✓ Complete"
    echo ""
done

echo "=== Quick Demo Complete ==="