# Script per generare automaticamente i tipi dal server OpenAPI
param(
    [string]$ServerUrl = "http://localhost:8002",
    [string]$OutputDir = "src/generated",
    [switch]$Clean = $false
)

Write-Host "🚀 Generazione tipi dal server OpenAPI..." -ForegroundColor Green

# Verifica che il server sia raggiungibile
$apiUrl = "$ServerUrl/api-docs/openapi.json"
Write-Host "🔍 Verifico server su: $apiUrl" -ForegroundColor Blue

try {
    $response = Invoke-RestMethod -Uri $apiUrl -Method Get -TimeoutSec 10
    Write-Host "✅ Server raggiungibile" -ForegroundColor Green
} catch {
    Write-Host "❌ Errore: Server non raggiungibile su $apiUrl" -ForegroundColor Red
    Write-Host "   Assicurati che il server sia avviato" -ForegroundColor Yellow
    exit 1
}

# Crea directory di output
if ($Clean -and (Test-Path $OutputDir)) {
    Write-Host "🧹 Pulizia directory esistente..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $OutputDir
}

if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# Verifica che openapi-generator sia installato
try {
    $version = & openapi-generator-cli version 2>$null
    Write-Host "✅ openapi-generator-cli trovato: $version" -ForegroundColor Green
} catch {
    Write-Host "❌ openapi-generator-cli non trovato" -ForegroundColor Red
    Write-Host "   Installa con: npm install -g @openapitools/openapi-generator-cli" -ForegroundColor Yellow
    
    # Offri di installarlo automaticamente
    $install = Read-Host "Vuoi installarlo ora? (y/N)"
    if ($install -eq "y" -or $install -eq "Y") {
        Write-Host "📦 Installazione openapi-generator-cli..." -ForegroundColor Blue
        npm install -g @openapitools/openapi-generator-cli
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Errore durante l'installazione" -ForegroundColor Red
            exit 1
        }
    } else {
        exit 1
    }
}

# Genera i tipi Rust
Write-Host "🔧 Generazione tipi Rust..." -ForegroundColor Blue

$configFile = "$OutputDir/config.json"
$rustConfig = @{
    packageName = "api_types"
    packageVersion = "1.0.0"
    packageAuthors = @("Generated")
    edition = "2021"
} | ConvertTo-Json

Set-Content -Path $configFile -Value $rustConfig

try {
    & openapi-generator-cli generate `
        -i $apiUrl `
        -g rust `
        -o "$OutputDir/rust" `
        -c $configFile `
        --additional-properties=supportAsync=true,preferUnsignedInt=true
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Tipi Rust generati con successo" -ForegroundColor Green
    } else {
        throw "Errore nella generazione"
    }
} catch {
    Write-Host "❌ Errore nella generazione tipi Rust: $_" -ForegroundColor Red
    exit 1
}

# Crea un modulo Rust per importare i tipi generati
$modFile = "$OutputDir/mod.rs"
$modContent = @"
//! Tipi generati automaticamente dal server OpenAPI
//! 
//! 🚀 Generato automaticamente da generate-types.ps1
//! ⚠️  Non modificare manualmente questo file
//! 
//! Per rigenerare: .\generate-types.ps1

pub use rust::apis::*;
pub use rust::models::*;

// Re-export dei tipi più comuni per comodità
pub mod auth {
    pub use super::rust::models::{UserLoginDto as LoginRequest, ApiSuccessResponseTokenReadDto as TokenResponse};
}

pub mod user {
    pub use super::rust::models::{UserRegisterDto as RegisterRequest, ApiSuccessResponseUserReadDto as UserProfile};
}
"@

Set-Content -Path $modFile -Value $modContent

Write-Host "✅ Generazione completata!" -ForegroundColor Green
Write-Host "📁 File generati in: $OutputDir" -ForegroundColor Blue
Write-Host "📋 Per usare i tipi: use crate::generated::auth::*;" -ForegroundColor Yellow

# Mostra un riassunto dei tipi generati
$modelsDir = "$OutputDir/rust/src/models"
if (Test-Path $modelsDir) {
    $models = Get-ChildItem $modelsDir -Filter "*.rs" | Select-Object -ExpandProperty BaseName
    Write-Host "`n📊 Tipi generati:" -ForegroundColor Cyan
    $models | ForEach-Object { Write-Host "   - $_" -ForegroundColor White }
}

Write-Host "`n🔄 Per aggiornare i tipi, riavvia il server e riesegui questo script" -ForegroundColor Green
