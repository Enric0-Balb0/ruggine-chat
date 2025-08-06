# Build or Watch Tailwind CSS for the client
param(
    [switch]$Watch
)

Write-Host "Building Tailwind CSS..." -ForegroundColor Green

# Check if npm is available
if (Get-Command npm -ErrorAction SilentlyContinue) {
    if ($Watch) {
        Write-Host "Starting Tailwind CSS watcher..." -ForegroundColor Yellow
        npm run watch-css
    } else {
        npm run build-css
        Write-Host "Tailwind CSS built successfully!" -ForegroundColor Green
    }
} else {
    # Fallback to npx if npm is not in PATH
    Write-Host "npm not found, trying npx..." -ForegroundColor Yellow
    if ($Watch) {
        Write-Host "Starting Tailwind CSS watcher with npx..." -ForegroundColor Yellow
        npx tailwindcss -i ./src/styles/index.css -o ./dist/tailwind.css --watch
    } else {
        npx tailwindcss -i ./src/styles/index.css -o ./dist/tailwind.css --minify
        Write-Host "Tailwind CSS built successfully with npx!" -ForegroundColor Green
    }
}
