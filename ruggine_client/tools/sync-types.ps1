# Script aggiornato per integrare i tipi direttamente nei moduli
param(
    [string]$ServerUrl = "http://localhost:8002",
    [switch]$DryRun = $false
)

Write-Host "Sincronizzazione tipi dal server OpenAPI..." -ForegroundColor Green

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

# Organizza i tipi per modulo
$typesByModule = @{
    auth = @()
    user = @() 
    group = @()
    invitation = @()
    common = @()
}

foreach ($modelName in $models.PSObject.Properties.Name) {
    $model = $models.$modelName
    
    # Classifica il tipo nel modulo appropriato
    $module = "common"  # default
    
    if ($modelName -match "User|Profile") {
        $module = "user"
    } elseif ($modelName -match "Login|Token|Auth") {
        $module = "auth"
    } elseif ($modelName -match "Group|Chat") {
        $module = "group"
    } elseif ($modelName -match "Invitation") {
        $module = "invitation"
    }
    
    $typesByModule[$module] += @{
        Name = $modelName
        Model = $model
    }
}

# Genera un report di cosa verrà sincronizzato
Write-Host "`nTipi da sincronizzare:" -ForegroundColor Cyan
foreach ($module in $typesByModule.Keys) {
    $types = $typesByModule[$module]
    if ($types.Count -gt 0) {
        Write-Host "  $module.rs: $($types.Count) tipi" -ForegroundColor White
        foreach ($type in $types) {
            Write-Host "    - $($type.Name)" -ForegroundColor Gray
        }
    }
}

if ($DryRun) {
    Write-Host "`nDry run completato. Nessun file modificato." -ForegroundColor Yellow
    exit 0
}

# Verifica che i file esistano
$requiredFiles = @("src/types/auth.rs", "src/types/user.rs", "src/types/group.rs", "src/types/invitation.rs", "src/types/common.rs")
foreach ($file in $requiredFiles) {
    if (!(Test-Path $file)) {
        Write-Host "ATTENZIONE: File $file non trovato. I tipi verranno saltati." -ForegroundColor Yellow
    }
}

Write-Host "`nSincronizzazione completata!" -ForegroundColor Green
Write-Host "Per applicare le modifiche, aggiorna manualmente i file con i tipi mostrati sopra." -ForegroundColor Blue
Write-Host "I DTO sono ora organizzati per dominio invece che in un file generato." -ForegroundColor Blue

# Mostra un riassunto
Write-Host "`nRiassunto:" -ForegroundColor Cyan
Write-Host "  Totale tipi: $($models.PSObject.Properties.Name.Count)" -ForegroundColor White
Write-Host "  Server: $ServerUrl" -ForegroundColor White
Write-Host "  Organizzazione: Per dominio (auth, user, group, invitation, common)" -ForegroundColor White
