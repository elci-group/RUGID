@echo off
REM RUGID RDF Demo Suite Runner (Windows Batch)
REM Quick demo sequence (1 minute)

if "%1"=="--help" (
    echo RUGID RDF Quick Demo Runner
    echo Usage: run_quick_demo.bat
    echo.
    echo Runs a sequence of RDF demos for quick verification.
    exit /b 0
)

echo === RUGID RDF Quick Demo (1 minute) ===
echo.

echo ▶ Running: 01_hello_world.rdf (5s)
start /B cargo run --quiet --bin rdf_render -- demos/rdf/01_hello_world.rdf
timeout /t 5 /nobreak >nul
taskkill /F /IM rdf_render.exe 2>nul
echo ✓ Complete
echo.

echo ▶ Running: 07_rotating_cube.rdf (12s)
start /B cargo run --quiet --bin rdf_render -- demos/rdf/07_rotating_cube.rdf
timeout /t 12 /nobreak >nul
taskkill /F /IM rdf_render.exe 2>nul
echo ✓ Complete
echo.

echo ▶ Running: 05_platonic_solids.rdf (8s)
start /B cargo run --quiet --bin rdf_render -- demos/rdf/05_platonic_solids.rdf
timeout /t 8 /nobreak >nul
taskkill /F /IM rdf_render.exe 2>nul
echo ✓ Complete
echo.

echo === Quick Demo Complete ===