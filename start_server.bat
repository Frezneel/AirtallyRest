@echo off
echo Starting AirTally REST API...
echo.

echo Checking database connection...
psql -U postgres -h localhost -p 5432 -c "\q" 2>nul
if %errorlevel% neq 0 (
    echo ERROR: PostgreSQL tidak dapat diakses. Pastikan PostgreSQL berjalan.
    echo Silakan jalankan PostgreSQL service terlebih dahulu.
    pause
    exit /b 1
)

echo Database connection OK.
echo.

echo Starting server...
cargo run

pause