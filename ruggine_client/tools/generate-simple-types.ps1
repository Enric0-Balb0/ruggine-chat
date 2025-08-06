# Script semplificato per generare tipi compatibili
param(
    [string]$ServerUrl = "http://localhost:8002",
    [string]$OutputFile = "src/types/generated.rs"
)

Write-Host "Generazione tipi semplificata dal server..." -ForegroundColor Green

# Scarica le specifiche OpenAPI
$apiUrl = "$ServerUrl/api-docs/openapi.json"
Write-Host "Scarico specifiche da: $apiUrl" -ForegroundColor Blue

try {
    $openapi = Invoke-RestMethod -Uri $apiUrl -Method Get
    Write-Host "Specifiche scaricate" -ForegroundColor Green
} catch {
    Write-Host "Errore: Server non raggiungibile" -ForegroundColor Red
    exit 1
}

# Estrai i modelli dalle specifiche
$models = $openapi.components.schemas

# Funzione per convertire tipi OpenAPI a Rust
function ConvertType {
    param($type, $format)
    
    switch ($type) {
        "string" {
            switch ($format) {
                "date-time" { return "DateTime<Utc>" }
                "date" { return "NaiveDate" }
                "uuid" { return "String" }
                default { return "String" }
            }
        }
        "integer" {
            switch ($format) {
                "int64" { return "i64" }
                "int32" { return "i32" }
                default { return "i32" }
            }
        }
        "number" {
            switch ($format) {
                "double" { return "f64" }
                "float" { return "f32" }
                default { return "f64" }
            }
        }
        "boolean" { return "bool" }
        "array" { return "Vec<UnknownType>" }
        default { return "serde_json::Value" }
    }
}

# Genera il contenuto del file
$content = @"
//! Tipi generati automaticamente dal server OpenAPI
//! 
//! Generato da: generate-simple-types.ps1
//! Generato il: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
//! Non modificare manualmente questo file

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

"@

# Genera le strutture
foreach ($modelName in $models.PSObject.Properties.Name) {
    $model = $models.$modelName
    
    # Gestisci enum
    if ($model.enum) {
        $content += "`n/// Enum $modelName`n"
        $content += "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`n"
        $content += "pub enum $modelName {`n"
        
        foreach ($enumValue in $model.enum) {
            $rustVariant = $enumValue -replace '([a-z])([A-Z])', '$1$2' # PascalCase
            $rustVariant = (Get-Culture).TextInfo.ToTitleCase($rustVariant.ToLower())
            $content += "    #[serde(rename = `"$enumValue`")]`n"
            $content += "    $rustVariant,`n"
        }
        
        $content += "}`n"
    }
    # Gestisci struct
    elseif ($model.type -eq "object" -and $model.properties) {
        $content += "`n/// $($model.description)`n"
        $content += "#[derive(Debug, Clone, Serialize, Deserialize)]`n"
        $content += "#[serde(rename_all = `"camelCase`")]`n"
        $content += "pub struct $modelName {`n"
        
        foreach ($propName in $model.properties.PSObject.Properties.Name) {
            $prop = $model.properties.$propName
            $rustType = ConvertType $prop.type $prop.format
            
            # Gestisci array
            if ($prop.type -eq "array" -and $prop.items) {
                $itemType = ConvertType $prop.items.type $prop.items.format
                $rustType = "Vec<$itemType>"
            }
            
            # Gestisci optional
            $isRequired = $model.required -contains $propName
            if (-not $isRequired) {
                $rustType = "Option<$rustType>"
            }
            
            $content += "    pub $($propName): $rustType,`n"
        }
        
        $content += "}`n"
    }
}

# Aggiungi alias per compatibilità
$content += @"


// Alias per compatibilita con il codice esistente
pub type LoginRequest = UserLoginDto;
pub type TokenResponse = ApiSuccessResponseTokenReadDto;
pub type UserProfile = ApiSuccessResponseUserReadDto;
pub type RegisterRequest = UserRegisterDto;
"@

# Crea directory se non esiste
$outputDir = Split-Path $OutputFile -Parent
if (!(Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

# Scrivi il file
Set-Content -Path $OutputFile -Value $content -Encoding UTF8

Write-Host "File generato: $OutputFile" -ForegroundColor Green
Write-Host "Modelli generati: $($models.PSObject.Properties.Name.Count)" -ForegroundColor Blue

# Mostra i modelli
Write-Host "`nModelli disponibili:" -ForegroundColor Cyan
$models.PSObject.Properties.Name | Sort-Object | ForEach-Object {
    Write-Host "   - $_" -ForegroundColor White
}
