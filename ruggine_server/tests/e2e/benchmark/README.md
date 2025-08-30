# Benchmark Tests

Questa directory contiene test di benchmark per valutare le prestazioni del server Ruggine sotto diversi carichi di lavoro.

## Setup Prerequisiti

Prima di eseguire i test di benchmark, assicurati di:

1. **Configurare il database di test:**
   ```bash
   # Assicurati che la variabile TEST_DATABASE_URL sia impostata
   export TEST_DATABASE_URL="postgres://testuser:testpass@localhost/ruggine_test"
   ```

2. **Configurare il logging CPU:**
   I test impostano automaticamente `LOG_IN_MILLISECONDS=50` per il monitoraggio delle prestazioni CPU.

## Test Disponibili

### 1. Message Load Test (`message_load_test.rs`)

Test di carico per la creazione di messaggi con utenti concorrenti.

**Test inclusi:**
- `test_concurrent_message_creation_benchmark`: 5 utenti creano 1000 messaggi ciascuno (5000 messaggi totali)
- `test_websocket_message_broadcast_benchmark`: 10 utenti inviano 100 messaggi via WebSocket
- `test_mixed_operations_benchmark`: 8 utenti eseguono 250 operazioni miste (HTTP API + queries)

**Per eseguire:**
```bash
cargo test test_concurrent_message_creation_benchmark -- --nocapture
cargo test test_websocket_message_broadcast_benchmark -- --nocapture  
cargo test test_mixed_operations_benchmark -- --nocapture
```

### 2. Database Stress Test (`database_stress_test.rs`)

Test di stress per il database e il connection pooling.

**Test inclusi:**
- `test_database_stress_benchmark`: 20 utenti eseguono 500 operazioni database intensive
- `test_connection_pool_stress_benchmark`: 100 connessioni concorrenti con 50 richieste ciascuna

**Per eseguire:**
```bash
cargo test test_database_stress_benchmark -- --nocapture
cargo test test_connection_pool_stress_benchmark -- --nocapture
```

### 3. Comprehensive Benchmark (`comprehensive_benchmark.rs`)

Suite completa che esegue scenari multipli in sequenza per una valutazione completa.

**Test inclusi:**
- `test_comprehensive_server_benchmark`: Esegue 4 scenari di carico crescente e genera un report completo

**Per eseguire:**
```bash
cargo test test_comprehensive_server_benchmark -- --nocapture
```

## Esecuzione Seriale vs Parallela

⚠️ **IMPORTANTE**: I test di benchmark dovrebbero essere eseguiti **SERIALMENTE** per ottenere risultati accurati e consistenti.

### Eseguire tutti i benchmark serialmente:
```bash
# Esegui tutti i test benchmark uno alla volta
cargo test benchmark -- --test-threads=1 --nocapture
```

### Eseguire test specifici:
```bash
# Test di carico messaggi
cargo test test_concurrent_message_creation_benchmark -- --test-threads=1 --nocapture

# Test stress database  
cargo test test_database_stress_benchmark -- --test-threads=1 --nocapture

# Suite completa
cargo test test_comprehensive_server_benchmark -- --test-threads=1 --nocapture
```

## Interpretazione dei Risultati

### Metriche Chiave

1. **Operations per Second (ops/sec)**: Throughput del server
2. **Success Rate (%)**: Affidabilità sotto carico
3. **CPU Usage (%)**: Utilizzo delle risorse del server
4. **Duration**: Tempo totale per completare il test

### Benchmarks di Riferimento

**Prestazioni Attese (valori orientativi):**
- Message creation: >200 ops/sec
- Database queries: >100 ops/sec  
- Success rate: >85% per tutti i test
- CPU usage: <80% durante picchi di carico

### Esempio Output

```
🎯 BENCHMARK SUMMARY: Heavy Load (8 users × 300 messages)
═══════════════════════════════════════════════════════
📊 Operations:
  • Total attempted: 2400
  • Successful: 2387
  • Failed: 13
  • Success rate: 99.46%
⏱️  Performance:
  • Duration: 12.456s
  • Operations/second: 191.62
  • Avg time/operation: 5.2ms
💻 CPU Usage:
  • Average: 45.23%
  • Maximum: 78.91%
  • Log entries: 249
═══════════════════════════════════════════════════════
```

## Cleanup e Gestione Risorse

I test sono progettati per:

1. **Cleanup automatico**: Tutti i dati di test vengono rimossi automaticamente
2. **Gestione CPU logs**: I log CPU vengono puliti prima e dopo ogni test
3. **Gestione utenti e gruppi**: Rimozione corretta da gruppi prima della cancellazione
4. **Gestione messaggi**: Tracciamento e rimozione degli ID dei messaggi creati

## Troubleshooting

### Errori Comuni

1. **Database connection errors**: Verifica che il database di test sia attivo e accessibile
2. **Port already in use**: Aspetta che i test precedenti terminino completamente
3. **Memory issues**: Esegui i test serialmente con `--test-threads=1`

### Debug

Per debug dettagliato, aggiungi logging:
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Performance Issues

Se i test falliscono per prestazioni:
1. Verifica le risorse di sistema disponibili
2. Controlla la configurazione del database
3. Riduci il numero di utenti/operazioni nei test per la tua configurazione

## Configurazione Personalizzata

Puoi modificare i parametri dei test editando le costanti nei file:

```rust
const NUM_USERS: usize = 5;           // Numero di utenti concorrenti
const MESSAGES_PER_USER: usize = 1000; // Messaggi per utente
const TOTAL_MESSAGES: usize = NUM_USERS * MESSAGES_PER_USER;
```

## Integrazione CI/CD

Per usare i benchmark in CI/CD:

```yaml
- name: Run benchmark tests
  run: |
    export LOG_IN_MILLISECONDS=50
    cargo test benchmark -- --test-threads=1 --nocapture
```
