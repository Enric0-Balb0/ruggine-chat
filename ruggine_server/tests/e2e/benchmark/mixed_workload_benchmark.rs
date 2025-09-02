use crate::common::{
    add_test_user_to_a_group, cleanup_all_cpu_usage_log, cleanup_group_chat,
    cleanup_test_user_from_a_group_chat, cleanup_text_messages,
    cleanup_user_by_email, create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    start_test_server
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ServerEvent, WebSocketMessage};
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[cfg(test)]
mod mixed_workload_benchmark_tests {
    use super::*;
    use crate::{common, create_admin_login_and_get_token, get_database};
    use futures::stream::{self, StreamExt};
    use futures_util::stream::FuturesUnordered;
    use rand::{Rng, SeedableRng};
    use reqwest::Client;
    use ruggine_server::config::parameter;
    use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use rand::rngs::StdRng;
    use tokio::sync::{Mutex, Semaphore};
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }

    #[derive(Debug)]
    struct BenchmarkStats {
        messages_created: AtomicUsize,
        not_read_queries: AtomicUsize,
        paginated_queries: AtomicUsize,
        mark_read_operations: AtomicUsize,
        websocket_passive_connections: AtomicUsize,  // Solo ricevono messaggi
        websocket_active_connections: AtomicUsize,   // Ricevono E rispondono
        websocket_mark_reads: AtomicUsize,
        failed_operations: AtomicUsize,
    }

    impl BenchmarkStats {
        fn new() -> Self {
            Self {
                messages_created: AtomicUsize::new(0),
                not_read_queries: AtomicUsize::new(0),
                paginated_queries: AtomicUsize::new(0),
                mark_read_operations: AtomicUsize::new(0),
                websocket_passive_connections: AtomicUsize::new(0),
                websocket_active_connections: AtomicUsize::new(0),
                websocket_mark_reads: AtomicUsize::new(0),
                failed_operations: AtomicUsize::new(0),
            }
        }

        fn print_stats(&self, duration: Duration) {
            let messages = self.messages_created.load(Ordering::SeqCst);
            let not_read = self.not_read_queries.load(Ordering::SeqCst);
            let paginated = self.paginated_queries.load(Ordering::SeqCst);
            let mark_read = self.mark_read_operations.load(Ordering::SeqCst);
            let ws_passive = self.websocket_passive_connections.load(Ordering::SeqCst);
            let ws_active = self.websocket_active_connections.load(Ordering::SeqCst);
            let ws_mark_reads = self.websocket_mark_reads.load(Ordering::SeqCst);
            let failures = self.failed_operations.load(Ordering::SeqCst);

            println!("\n📊 RISULTATI BENCHMARK MIXED WORKLOAD:");
            println!("=== DURATA TEST: {:?} ===", duration);
            println!("  📝 Messaggi creati: {}", messages);
            println!("  🔍 Query messaggi non letti: {}", not_read);
            println!("  📄 Query messaggi paginati: {}", paginated);
            println!("  ✅ Operazioni mark as read: {}", mark_read);
            println!("  🔌 Connessioni WebSocket passive: {}", ws_passive);
            println!("  🔌 Connessioni WebSocket attive: {}", ws_active);
            println!("  📨 Mark as read da WebSocket: {}", ws_mark_reads);
            println!("  ❌ Operazioni fallite: {}", failures);
            
            let total_ops = messages + not_read + paginated + mark_read + ws_mark_reads;
            println!("  🎯 Operazioni totali: {}", total_ops);
            println!("  📈 Ops/sec: {:.2}", total_ops as f64 / duration.as_secs_f64());
            println!("  📉 Failure rate: {:.2}%", failures as f64 / total_ops.max(1) as f64 * 100.0);
        }
    }

    /// Test di benchmark con carico di lavoro misto: creazione messaggi, letture, websocket
    #[tokio::test(flavor = "multi_thread", worker_threads = 32)]
    #[serial]
    async fn test_mixed_workload_benchmark() {
        // Cleanup iniziale
        // init_tracing();
        cleanup_all_cpu_usage_log().await;

        let db = get_database().await;
        let mut cpu_usage_log_service = CpuUsageLogService::new(
            Arc::new(CpuUsageLogRepository::new(&db))
        );
        cpu_usage_log_service.set_monitoring_interval_ms(1000);
        cpu_usage_log_service.start_monitoring().await.expect("Failed to start CPU monitoring");
        let mut rng = StdRng::from_entropy();

        // Parametri del benchmark
        // ---------------- Parametri generali ----------------
        const NUM_GROUPS: usize = 450;
        const TEST_DURATION_SECS: u64 = 130;

        // ---------------- Utenti per gruppo ----------------
        const MESSAGE_SENDERS_PER_GROUP: usize = 2;    // Utenti che inviano messaggi
        const NOT_READ_READERS_PER_GROUP: usize = 2;   // Utenti che leggono e marcano messaggi
        const PAGINATED_READERS_PER_GROUP: usize = 2; // Utenti che leggono paginated
        const WEBSOCKET_USERS_PER_GROUP: usize = 8;   // Utenti connessi al websocket
        const WEBSOCKET_PASSIVE_USERS: usize = 8;     // Utenti che solo ricevono messaggi
        const WEBSOCKET_ACTIVE_USERS: usize = 1;      // Utenti che ricevono E rispondono con update_read_at

        // ---------------- Intervalli in ms (con jitter) ----------------
        // Messaggi: ogni 25–35s
        const INTERVAL_SENDERS_PER_GROUP: usize = 30000;
        const DELTA_INTERVAL_SENDERS_PER_GROUP: usize = 5000;

        // Not-read readers: ogni 20–40s
        const INTERVAL_NOT_READ_PER_GROUP: usize = 30000;
        const DELTA_INTERVAL_NOT_READ_PER_GROUP: usize = 10000;

        // Paginated readers: ogni 40–70s
        const INTERVAL_PAGINATED_PER_GROUP: usize = 55000;
        const DELTA_INTERVAL_PAGINATED_PER_GROUP: usize = 15000;

        // Websocket heartbeat / update online: ogni 1–3s
        const INTERVAL_WEBSOCKET_PER_GROUP: usize = 2000;
        const DELTA_INTERVAL_WEBSOCKET_PER_GROUP: usize = 1000;

        // ---------------- Funzione helper per sleep con jitter ----------------
        fn jitter(base: usize, delta: usize, rng: &mut StdRng) -> Duration {
            let variation = rng.gen_range(-(delta as i64)..=(delta as i64));
            Duration::from_millis((base as i64 + variation).max(0) as u64)
        }

        info!("🚀 Benchmark Mixed Workload: {} gruppi × {} senders × {} not-read × {} paginated × {} websocket ({}+{}) per {} secondi", 
              NUM_GROUPS, MESSAGE_SENDERS_PER_GROUP, NOT_READ_READERS_PER_GROUP, PAGINATED_READERS_PER_GROUP, 
              WEBSOCKET_USERS_PER_GROUP, WEBSOCKET_PASSIVE_USERS, WEBSOCKET_ACTIVE_USERS, TEST_DURATION_SECS);

        // Avvio server di test
        let (addr, shutdown) = start_test_server().await;
        let base_url = format!("http://{}", addr);

        // --- Creazione utenti per ogni gruppo ---
        let mut all_users = Vec::new();
        let mut all_tokens = Vec::new();
        let mut groups = Vec::new();

        let users_per_group = MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP + PAGINATED_READERS_PER_GROUP + WEBSOCKET_USERS_PER_GROUP;

        for g in 0..NUM_GROUPS {
            // Creazione utenti per questo gruppo
            let mut group_users = Vec::new();
            let mut group_tokens = Vec::new();

            for u in 0..users_per_group {
                let (user, _, token) = create_login_and_get_token(format!("bench_g{}_u{}", g, u)).await;
                group_users.push(user);
                group_tokens.push(token);
            }

            // Creazione gruppo con il primo utente come proprietario
            let group = create_test_group_chat_with_invitation_and_membership(
                format!("bench_group_{}", g).as_str(),
                group_users[0].id
            ).await;

            // Aggiunta di tutti gli altri utenti al gruppo
            for user in &group_users[1..] {
                add_test_user_to_a_group(user.id, &group).await;
            }

            all_users.extend(group_users);
            all_tokens.extend(group_tokens);
            groups.push(group);
        }

        println!("✅ Creati {} utenti in {} gruppi", all_users.len(), NUM_GROUPS);

        // Strutture dati condivise
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(BenchmarkStats::new());
        let created_messages: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

        // --- Avvio task WebSocket users (devono connettersi prima) ---
        let mut websocket_handles = Vec::new();
        let mut user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group = &groups[g];
            let ws_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP + PAGINATED_READERS_PER_GROUP;

            // WebSocket passive users (solo ricevono messaggi)
            for ws_user_idx in ws_start_idx..ws_start_idx + WEBSOCKET_PASSIVE_USERS {
                let token = all_tokens[ws_user_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let group_id = group.id;
                let mut rng = rng.clone();

                let handle = tokio::spawn(async move {
                    // Tentativo di connessione al WebSocket
                    let ws_url = format!("ws://{}/api/ws/chat?token={}", addr, token);
                    let connect_result = connect_async(&ws_url).await;
                    
                    let (mut ws_stream, _) = match connect_result {
                        Ok(result) => result,
                        Err(e) => {
                            println!("Failed websocket passive connection got {:?}", e);
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                            return;
                        }
                    };

                    stats.websocket_passive_connections.fetch_add(1, Ordering::SeqCst);

                    // Join del gruppo
                    let join_id = Uuid::new_v4().to_string();
                    let join_msg = WebSocketMessage::Request {
                        request_id: join_id,
                        action: ClientAction::Groups(GroupAction::Join {}),
                    };

                    if let Ok(msg_json) = join_msg.to_json() {
                        if let Err(e) = ws_stream.send(Message::Text(msg_json)).await {
                            println!("Failed websocket passive join got {:?}", e);
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                            return;
                        }
                    }

                    // Loop di ascolto per NewMessage (senza rispondere)
                    while !stop_flag.load(Ordering::SeqCst) {
                        let timeout_result = timeout(jitter(INTERVAL_WEBSOCKET_PER_GROUP, DELTA_INTERVAL_WEBSOCKET_PER_GROUP, &mut rng), ws_stream.next()).await;
                        
                        if let Ok(Some(Ok(Message::Text(text)))) = timeout_result {
                            if let Ok(ws_msg) = WebSocketMessage::from_json(&text) {
                                if let WebSocketMessage::Event { event: ServerEvent::Groups(GroupEvent::NewMessage { .. }), .. } = ws_msg {
                                    // Solo riceve il messaggio, non fa nulla
                                }
                            }
                        }
                    }
                });

                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                websocket_handles.push(handle);
            }

            // WebSocket active users (ricevono E rispondono con update_read_at)
            for ws_user_idx in ws_start_idx + WEBSOCKET_PASSIVE_USERS..ws_start_idx + WEBSOCKET_USERS_PER_GROUP {
                let token = all_tokens[ws_user_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
                let group_id = group.id;
                let mut rng = rng.clone();

                let handle = tokio::spawn(async move {
                    // Tentativo di connessione al WebSocket
                    let ws_url = format!("ws://{}/api/ws/chat?token={}", addr, token);
                    let connect_result = connect_async(&ws_url).await;
                    
                    let (mut ws_stream, _) = match connect_result {
                        Ok(result) => result,
                        Err(e) => {
                            println!("Failed websocket active connection got {:?}", e);
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                            return;
                        }
                    };

                    stats.websocket_active_connections.fetch_add(1, Ordering::SeqCst);

                    // Join del gruppo
                    let join_id = Uuid::new_v4().to_string();
                    let join_msg = WebSocketMessage::Request {
                        request_id: join_id,
                        action: ClientAction::Groups(GroupAction::Join {}),
                    };

                    if let Ok(msg_json) = join_msg.to_json() {
                        if let Err(e) = ws_stream.send(Message::Text(msg_json)).await {
                            println!("Failed websocket active join got {:?}", e);
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                            return;
                        }
                    }

                    // Loop di ascolto per NewMessage con risposta update_read_at
                    while !stop_flag.load(Ordering::SeqCst) {
                        let timeout_result = timeout(jitter(INTERVAL_WEBSOCKET_PER_GROUP, DELTA_INTERVAL_WEBSOCKET_PER_GROUP, &mut rng), ws_stream.next()).await;
                        
                        if let Ok(Some(Ok(Message::Text(text)))) = timeout_result {
                            if let Ok(ws_msg) = WebSocketMessage::from_json(&text) {
                                if let WebSocketMessage::Event { event: ServerEvent::Groups(GroupEvent::NewMessage { message_id, .. }), .. } = ws_msg {
                                    // Simula mark as read del messaggio ricevuto
                                    let client = Client::new();
                                    let payload = json!({
                                        "text_message_id": message_id,
                                        "read_at": Utc::now()
                                    });

                                    let mark_read_result = client
                                        .patch(&format!("{}/api/text_message/update_read_at", base_url))
                                        .bearer_auth(&token)
                                        .json(&payload)
                                        .send()
                                        .await;

                                    if let Ok(resp) = mark_read_result {
                                        if resp.status().is_success() {
                                            stats.websocket_mark_reads.fetch_add(1, Ordering::SeqCst);
                                        } else {
                                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                            let status = resp.status();
                                            match resp.text().await {
                                                Ok(body) => {
                                                    stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                                    println!("Mark read response failed: status={}, body={}", status, body);
                                                }
                                                Err(e) => {
                                                    stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                                    println!("Mark read response failed: status={}, could not read body ({:?})", status, e);
                                                }
                                            }
                                        }
                                    } else if let Err(e) = mark_read_result {
                                        stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                        println!("Mark read request error: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                });

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                websocket_handles.push(handle);
            }

            user_idx += users_per_group;
        }

        // Pausa per permettere le connessioni WebSocket
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let start_time = Instant::now();

        // --- Avvio task NOT READ READERS ---
        let mut not_read_handles = Vec::new();
        user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group = &groups[g];
            let readers_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP;

            for reader_idx in readers_start_idx..readers_start_idx + NOT_READ_READERS_PER_GROUP {
                let token = all_tokens[reader_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
                let group_id = group.id;
                let mut rng = rng.clone();

                let handle = tokio::spawn(async move {
                    let client = Client::new();

                    while !stop_flag.load(Ordering::SeqCst) {
                        tokio::time::sleep(jitter(INTERVAL_NOT_READ_PER_GROUP, DELTA_INTERVAL_NOT_READ_PER_GROUP, &mut rng)).await;
                        // Find not read messages
                        let not_read_result = client
                            .get(&format!("{}/api/text_message/group/{}/messages/not-read-yet", base_url, group_id))
                            .bearer_auth(&token)
                            .send()
                            .await;

                        match not_read_result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    stats.not_read_queries.fetch_add(1, Ordering::SeqCst);

                                    match response.json::<serde_json::Value>().await {
                                        Ok(json) => {
                                            if let Some(messages) = json.get("data").and_then(|d| d.as_array()) {
                                                // Mark random messages as read
                                                for message in messages.iter().take(3) { // Max 3 per iterazione
                                                    if let Some(msg_id) = message.get("id").and_then(|id| id.as_i64()) {
                                                        let payload = json!({
                                                            "text_message_id": msg_id,
                                                            "read_at": Utc::now()
                                                        });

                                                        let mark_result = client
                                                            .patch(&format!("{}/api/text_message/update_read_at", base_url))
                                                            .bearer_auth(&token)
                                                            .json(&payload)
                                                            .send()
                                                            .await;

                                                        match mark_result {
                                                            Ok(resp) => {
                                                                if resp.status().is_success() {
                                                                    stats.mark_read_operations.fetch_add(1, Ordering::SeqCst);
                                                                } else {
                                                                    let status = resp.status();
                                                                    match resp.text().await {
                                                                        Ok(body) => {
                                                                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                                                            println!("Mark result not successful: status={}, body={}", status, body);
                                                                        }
                                                                        Err(e) => {
                                                                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                                                            println!("Mark result not successful: status={}, could not read body ({:?})", status, e);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                                                println!("Mark result error: {:?}", e);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                            println!("Failed to parse JSON response: {:?}", e);
                                        }
                                    }
                                } else {
                                    stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                    println!("Not-read query failed with status: {:?}", response.status());
                                }
                            }
                            Err(e) => {
                                stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                                println!("Not-read request error: {:?}", e);
                            }
                        }
                    }
                });

                not_read_handles.push(handle);
            }

            user_idx += users_per_group;
        }

        // --- Avvio task PAGINATED READERS ---
        let mut paginated_handles = Vec::new();
        user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group = &groups[g];
            let readers_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP;

            for reader_idx in readers_start_idx..readers_start_idx + PAGINATED_READERS_PER_GROUP {
                let token = all_tokens[reader_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
                let group_id = group.id;
                let mut rng = rng.clone();

                let client = Client::new();
                let handle = tokio::spawn(async move {
                    while !stop_flag.load(Ordering::SeqCst) {
                        // Find paginated messages
                        tokio::time::sleep(jitter(INTERVAL_PAGINATED_PER_GROUP, DELTA_INTERVAL_PAGINATED_PER_GROUP, &mut rng)).await;
                        let paginated_result = client
                            .get(&format!("{}/api/text_message/group/{}/messages?limit=50", base_url, group_id))
                            .bearer_auth(&token)
                            .send()
                            .await;

                        if paginated_result.is_ok() && paginated_result.unwrap().status().is_success() {
                            stats.paginated_queries.fetch_add(1, Ordering::SeqCst);
                        } else {
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                });

                paginated_handles.push(handle);
            }

            user_idx += users_per_group;
        }

        // --- Avvio task MESSAGE SENDERS ---
        let mut sender_handles = Vec::new();
        user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group = &groups[g];

            for sender_idx in user_idx..user_idx + MESSAGE_SENDERS_PER_GROUP {
                let token = all_tokens[sender_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let created_messages = Arc::clone(&created_messages);
                let base_url = base_url.clone();
                let group_id = group.id;
                let mut rng = rng.clone();

                let handle = tokio::spawn(async move {
                    let client = Client::new();
                    let mut message_counter = 0;

                    while !stop_flag.load(Ordering::SeqCst) {
                        tokio::time::sleep(jitter(INTERVAL_SENDERS_PER_GROUP, DELTA_INTERVAL_SENDERS_PER_GROUP, &mut rng)).await;
                        message_counter += 1;
                        let payload = json!({
                            "content": format!("Benchmark message {} from sender {}", message_counter, sender_idx),
                            "group_chat_id": group_id
                        });

                        let result = client
                            .post(&format!("{}/api/text_message/create", base_url))
                            .bearer_auth(&token)
                            .json(&payload)
                            .send()
                            .await;

                        if let Ok(response) = result {
                            if response.status().is_success() {
                                stats.messages_created.fetch_add(1, Ordering::SeqCst);

                                if let Ok(json) = response.json::<serde_json::Value>().await {
                                    if let Some(msg_id) = json.get("data").and_then(|d| d.get("id")).and_then(|id| id.as_i64()) {
                                        created_messages.lock().await.push(msg_id as i32);
                                    }
                                }
                            } else {
                                stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                            }
                        } else {
                            stats.failed_operations.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                });

                sender_handles.push(handle);
            }

            user_idx += users_per_group;
        }

        println!(
            "🎯 Tutti i task avviati alle {} UTC, benchmark in corso per {} secondi...",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            TEST_DURATION_SECS
        );

        // Esecuzione del benchmark per la durata specificata
        tokio::time::sleep(tokio::time::Duration::from_secs(TEST_DURATION_SECS)).await;

        // Stop di tutti i task
        stop_flag.store(true, Ordering::SeqCst);

        // Attesa completamento di tutti i task
        futures::future::join_all(sender_handles).await;
        futures::future::join_all(not_read_handles).await;
        futures::future::join_all(paginated_handles).await;
        futures::future::join_all(websocket_handles).await;

        let total_duration = start_time.elapsed();

        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop CPU monitoring");

        println!("==============================");
        println!("🏁 Parametri del benchmark");
        println!("Numero di gruppi: {}", NUM_GROUPS);
        println!("Utenti che creano messaggi per gruppo: {}", MESSAGE_SENDERS_PER_GROUP);
        println!("Utenti che chiamano find not read + mark read per gruppo: {}", NOT_READ_READERS_PER_GROUP);
        println!("Utenti che chiamano find paginated per gruppo: {}", PAGINATED_READERS_PER_GROUP);
        println!("Utenti connessi al websocket (passivi) per gruppo: {}", WEBSOCKET_PASSIVE_USERS);
        println!("Utenti connessi al websocket (attivi con mark read) per gruppo: {}", WEBSOCKET_ACTIVE_USERS);
        println!("Durata del test (s): {}", TEST_DURATION_SECS);
        println!("Intervallo senders (ms): {} ± {}", INTERVAL_SENDERS_PER_GROUP, DELTA_INTERVAL_SENDERS_PER_GROUP);
        println!("Intervallo not read readers (ms): {} ± {}", INTERVAL_NOT_READ_PER_GROUP, DELTA_INTERVAL_NOT_READ_PER_GROUP);
        println!("Intervallo paginated readers (ms): {} ± {}", INTERVAL_PAGINATED_PER_GROUP, DELTA_INTERVAL_PAGINATED_PER_GROUP);
        println!("==============================\n");

        // Stampa statistiche finali
        stats.print_stats(total_duration);

        // Cleanup delle info dei messaggi e messaggi creati
        /* let message_ids = created_messages.lock().await.clone();
        println!("🧹 Cleanup di {} messaggi creati...", message_ids.len());

        cleanup_text_messages(message_ids.clone()).await;

        // Cleanup utenti e gruppi
        for g in 0..NUM_GROUPS {
            let user_start_idx = g * users_per_group;
            let user_end_idx = user_start_idx + users_per_group;
            let group = &groups[g];

            for user in &all_users[user_start_idx..user_end_idx] {
                cleanup_test_user_from_a_group_chat(user.id, group.id).await;
            }
            cleanup_group_chat(group.id).await;
        }

        for user in &all_users {
            cleanup_user_by_email(user.email.clone()).await;
        } */

        let _ = shutdown.send(());

        // Verifiche finali
        let total_ops = stats.messages_created.load(Ordering::SeqCst) + 
                       stats.not_read_queries.load(Ordering::SeqCst) + 
                       stats.paginated_queries.load(Ordering::SeqCst) + 
                       stats.mark_read_operations.load(Ordering::SeqCst) + 
                       stats.websocket_mark_reads.load(Ordering::SeqCst);

        let failure_rate = stats.failed_operations.load(Ordering::SeqCst) as f64 / total_ops.max(1) as f64;

        assert!(total_ops > 0, "Nessuna operazione completata durante il benchmark");
        assert!(failure_rate < 0.1, "Failure rate troppo alto: {:.2}%", failure_rate * 100.0);
        assert!(stats.websocket_passive_connections.load(Ordering::SeqCst) + stats.websocket_active_connections.load(Ordering::SeqCst) > 0, 
                "Nessuna connessione WebSocket stabilita");

        println!("✅ Benchmark completato con successo!");
    }
}
