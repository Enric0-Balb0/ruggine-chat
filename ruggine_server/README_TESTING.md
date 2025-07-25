# Testing Strategy for Ruggine Server

## Overview
Questo documento descrive la strategia di testing per il progetto Ruggine, progettato per team di sviluppo grandi e collaborativi.

## Struttura dei Test

```
ruggine_server/
├── src/                           # Codice sorgente
├── tests/
│   ├── common/                    # Utilities comuni per i test
│   │   └── mod.rs
│   ├── unit/                      # Test unitari
│   │   ├── repository/
│   │   ├── service/
│   │   ├── handler/
│   │   └── ...
│   └── integration/               # Test di integrazione
│       ├── repository/
│       ├── api/
│       └── ...
└── Cargo.toml
```

## Tipi di Test

### 1. Unit Tests (`tests/unit/`)
- **Scopo**: Testano singole unità di codice in isolamento
- **Caratteristiche**:
  - Veloci da eseguire
  - Non richiedono database o risorse esterne
  - Usano mock e stub
  - Si concentrano sulla logica business

**Esempio di esecuzione**:
```bash
cargo test --test "unit/*"
```

### 2. Integration Tests (`tests/integration/`)
- **Scopo**: Testano l'interazione tra componenti
- **Caratteristiche**:
  - Richiedono database di test
  - Testano flussi completi
  - Più lenti degli unit test
  - Verificano contratti tra moduli

**Esempio di esecuzione**:
```bash
cargo test --test "integration/*"
```

## Convenzioni di Naming

### File di Test
- Unit tests: `{module_name}_test.rs`
- Integration tests: `{module_name}_integration_test.rs`

### Funzioni di Test
- Unit tests: `test_{functionality_being_tested}`
- Integration tests: `test_{scenario_being_tested}`
- Error tests: `test_{error_condition}`
- Performance tests: `test_{performance_aspect}`

### Test Modules
```rust
#[cfg(test)]
mod {module_name}_unit_tests {
    // unit tests here
}

#[cfg(test)]
mod {module_name}_integration_tests {
    // integration tests here
}

#[cfg(test)]
mod {module_name}_error_tests {
    // error handling tests here
}
```

## Setup del Database di Test

### Variabili d'Ambiente
```bash
# Database di test (separato da quello di produzione)
export TEST_DATABASE_URL="mysql://root:password@localhost/ruggine_test"
```

### Docker per Test Database
```yaml
# docker-compose.test.yml
version: '3.8'
services:
  test_db:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: password
      MYSQL_DATABASE: ruggine_test
    ports:
      - "3307:3306"  # Porta diversa da produzione
```

## Comandi Utili

### Eseguire tutti i test
```bash
cargo test
```

### Eseguire solo unit tests
```bash
cargo test --test "*unit*"
```

### Eseguire solo integration tests
```bash
cargo test --test "*integration*" -- --test-threads=1
```

### Eseguire test con output dettagliato
```bash
cargo test -- --nocapture
```

### Eseguire test specifici
```bash
cargo test test_insert_and_find_by_email
```

### Coverage report
```bash
cargo tarpaulin --out Html
```

## Best Practices per Team Grandi

### 1. Isolamento dei Test
- Ogni test deve essere indipendente
- Usare `cleanup_test_db()` dopo ogni test di integrazione
- Non fare affidamento sull'ordine di esecuzione

### 2. Performance
- Unit tests devono essere < 100ms
- Integration tests devono essere < 5s
- Usare `#[ignore]` per test lenti durante sviluppo

### 3. Mocking
```rust
// Esempio di mock per dependencies
#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::mock;

mock! {
    MyDatabase {}
    
    #[async_trait]
    impl DatabaseTrait for MyDatabase {
        async fn get_pool(&self) -> &Pool<MySql>;
    }
}
```

### 4. Test Data Management
- Usare prefissi consistenti per test data (`test_`, `@test.com`)
- Implementare helper per creare test data
- Cleanup automatico in `Drop` trait se necessario

### 5. Parallel Execution
```rust
// Per test che non possono essere paralleli
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_that_needs_isolation() {
    // test code
}
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: password
          MYSQL_DATABASE: ruggine_test
        ports:
          - 3306:3306
    
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Unit Tests
        run: cargo test --test "*unit*"
      
      - name: Run Integration Tests
        run: cargo test --test "*integration*"
        env:
          TEST_DATABASE_URL: mysql://root:password@localhost/ruggine_test
```

## Metrics e Reporting

### Coverage Target
- Unit tests: > 90%
- Integration tests: > 80%
- Overall: > 85%

### Performance Benchmarks
- Unit test suite: < 30s
- Integration test suite: < 5min
- Full test suite: < 10min

## Troubleshooting

### Test Database Issues
```bash
# Reset test database
mysql -u root -p -e "DROP DATABASE IF EXISTS ruggine_test; CREATE DATABASE ruggine_test;"

# Run seeds
sqlx migrate run --database-url="mysql://root:password@localhost/ruggine_test"
```

### Common Test Failures
1. **Database connection**: Verificare TEST_DATABASE_URL
2. **Parallel execution**: Usare `--test-threads=1` per debugging
3. **Cleanup issues**: Verificare che cleanup_test_db() sia chiamato
4. **Timing issues**: Aggiungere appropriate await/timeout

## Team Workflow

### Pre-commit
```bash
# Eseguire prima di ogni commit
cargo test --test "*unit*"
cargo clippy
cargo fmt
```

### Pre-merge
```bash
# Eseguire prima di merge su main
cargo test
cargo tarpaulin
```

### Continuous Integration
- Tutti i test devono passare per il merge
- Coverage non deve diminuire
- Performance benchmarks devono essere rispettati
