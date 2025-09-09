use crate::common::{
    add_test_user_to_a_group, cleanup_all_cpu_usage_log, cleanup_group_chat,
    cleanup_test_user_from_a_group_chat, cleanup_text_messages,
    cleanup_user_by_email, create_login_and_get_token, create_test_group_chat_with_invitation_and_membership
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ServerEvent, WebSocketMessage};
use ruggine_server::factory::{user_factory::UserFactory, group_chat_factory::GroupChatFactory};
use ruggine_server::entity::invitation::InvitationStatus;
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
    use crate::{common, create_admin_login_and_get_token}; // get_database commented out for Docker mode
    use futures::stream::{self, StreamExt};
    use futures_util::stream::FuturesUnordered;
    use rand::{Rng, SeedableRng};
    use reqwest::Client;
    use ruggine_server::config::parameter;
    // NOTE: CPU monitoring imports disabled for Docker mode
    // use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    // use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    // use ruggine_server::utils::service_initializer::ServiceInitializer;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use chrono::DateTime;
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
        users_registered: AtomicUsize,
        users_login: AtomicUsize,
        groups_created: AtomicUsize,
        invitations_created: AtomicUsize,
        update_invitation_status: AtomicUsize,
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
                users_registered: AtomicUsize::new(0),
                users_login: AtomicUsize::new(0),
                groups_created: AtomicUsize::new(0),
                invitations_created: AtomicUsize::new(0),
                update_invitation_status: AtomicUsize::new(0),
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
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_mixed_workload_benchmark() {
        // Configuration: Use environment variable or default to Docker production port
        let server_url = std::env::var("BENCHMARK_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8002".to_string());
        let server_host = std::env::var("BENCHMARK_SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1:8002".to_string());
        
        println!("🎯 Benchmark configurazione:");
        println!("  Server URL (HTTP): {}", server_url);
        println!("  Server Host (WebSocket): {}", server_host);
        println!("  Per cambiare, imposta BENCHMARK_SERVER_URL e BENCHMARK_SERVER_HOST");
        println!();

        // Cleanup iniziale
        // init_tracing();
        cleanup_all_cpu_usage_log().await;

        // NOTE: CPU monitoring disabled when using Docker server
        // let db = get_database().await;
        // let mut cpu_usage_log_service = CpuUsageLogService::new(
        //     Arc::new(CpuUsageLogRepository::new(&db))
        // );
        // cpu_usage_log_service.set_monitoring_interval_ms(1000);
        // cpu_usage_log_service.start_monitoring().await.expect("Failed to start CPU monitoring");
        let mut rng = StdRng::from_entropy();

        // Parametri del benchmark
        // ---------------- Parametri generali ----------------
        const NUM_GROUPS: usize = 600; // Numero di gruppi da creare
        const TEST_DURATION_SECS: u64 = 130;

        // ---------------- Utenti per gruppo ----------------
        const MESSAGE_SENDERS_PER_GROUP: usize = 2;    // Utenti che inviano messaggi
        const NOT_READ_READERS_PER_GROUP: usize = 2;   // Utenti che leggono e marcano messaggi
        const PAGINATED_READERS_PER_GROUP: usize = 2; // Utenti che leggono paginated
        const WEBSOCKET_USERS_PER_GROUP: usize = 8;   // Utenti connessi al websocket
        const WEBSOCKET_PASSIVE_USERS: usize = 7;     // Utenti che solo ricevono messaggi
        const WEBSOCKET_ACTIVE_USERS: usize = 1;      // Utenti che ricevono E rispondono con update_read_at

        // ---------------- Intervalli in ms (con jitter) ----------------
        // Messaggi: ogni 25–35s
        const INTERVAL_SENDERS_PER_GROUP: usize = 30000;
        const DELTA_INTERVAL_SENDERS_PER_GROUP: usize = 30000;

        // Not-read readers: ogni 20–40s
        const INTERVAL_NOT_READ_PER_GROUP: usize = 30000;
        const DELTA_INTERVAL_NOT_READ_PER_GROUP: usize = 30000;

        // Paginated readers: ogni 40–70s
        const INTERVAL_PAGINATED_PER_GROUP: usize = 55000;
        const DELTA_INTERVAL_PAGINATED_PER_GROUP: usize = 55000;

        // Websocket heartbeat / update online: ogni 1–3s
        const INTERVAL_WEBSOCKET_PER_GROUP: usize = 2000;
        const DELTA_INTERVAL_WEBSOCKET_PER_GROUP: usize = 2000;

        // ---------------- Funzione helper per sleep con jitter ----------------
        fn jitter(base: usize, delta: usize, rng: &mut StdRng) -> Duration {
            let variation = rng.gen_range(-(delta as i64)..=(delta as i64));
            Duration::from_millis((base as i64 + variation).max(0) as u64)
        }

        info!("🚀 Benchmark Mixed Workload: {} gruppi × {} senders × {} not-read × {} paginated × {} websocket ({}+{}) per {} secondi", 
              NUM_GROUPS, MESSAGE_SENDERS_PER_GROUP, NOT_READ_READERS_PER_GROUP, PAGINATED_READERS_PER_GROUP, 
              WEBSOCKET_USERS_PER_GROUP, WEBSOCKET_PASSIVE_USERS, WEBSOCKET_ACTIVE_USERS, TEST_DURATION_SECS);

        // Connect to Docker server instead of starting test server
        let base_url = server_url;

        println!("🐳 Connecting to Docker server at: {}", base_url);

        // --- Creazione utenti e gruppi tramite API ---
        let client = Client::new();
        let mut all_users = Vec::new();
        let mut all_tokens = Vec::new(); 
        let mut groups = Vec::new();

        let users_per_group = MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP + PAGINATED_READERS_PER_GROUP + WEBSOCKET_USERS_PER_GROUP;

        let stats = Arc::new(BenchmarkStats::new());

        let start_time = Instant::now();

        info!("🚀 Creazione di {} utenti e {} gruppi tramite API...", NUM_GROUPS * users_per_group, NUM_GROUPS);

        // Task paralleli per la creazione di gruppi e utenti
        let mut group_creation_tasks = Vec::new();
        
        for g in 0..NUM_GROUPS {
            let base_url = base_url.clone();
            let client = client.clone();
            let stats = stats.clone();
            
            let task = tokio::spawn(async move {
                let mut group_users = Vec::new();
                let mut group_tokens = Vec::new();
                
                // Creazione utenti per questo gruppo
                for u in 0..users_per_group {
                    let user_dto = UserFactory::unique_fake_user_register_dto(&format!("bench_g{}_u{}", g, u));
                    
                    // Registrazione utente
                    let register_payload = json!({
                        "email": user_dto.email,
                        "password": user_dto.password,
                        "username": user_dto.username,
                        "first_name": user_dto.first_name,
                        "last_name": user_dto.last_name,
                        "birthday": user_dto.birthday.format("%Y-%m-%d").to_string(),
                        "address": user_dto.address,
                        "gender": user_dto.gender
                    });

                    let register_response = client
                        .post(&format!("{}/api/user/register", base_url))
                        .json(&register_payload)
                        .send()
                        .await
                        .expect("Failed to register user");

                    if !register_response.status().is_success() {
                        panic!("User registration failed for group {} user {}: {}", g, u, register_response.status());
                    }

                    stats.users_registered.fetch_add(1, Ordering::SeqCst);

                    let register_data: serde_json::Value = register_response.json().await
                        .expect("Failed to parse register response");

                    // Login utente 
                    let login_payload = json!({
                        "email": user_dto.email,
                        "password": user_dto.password
                    });

                    let login_response = client
                        .post(&format!("{}/api/auth/login", base_url))
                        .json(&login_payload)
                        .send()
                        .await
                        .expect("Failed to login user");

                    if !login_response.status().is_success() {
                        panic!("User login failed for group {} user {}: {}", g, u, login_response.status());
                    }

                    let login_data: serde_json::Value = login_response.json().await
                        .expect("Failed to parse login response");

                    stats.users_login.fetch_add(1, Ordering::SeqCst);

                    let token = login_data["data"]["token"].as_str().unwrap().to_string();
                    let user_data = &register_data["data"];
                    
                    group_users.push(user_data.clone());
                    group_tokens.push(token);
                }

                // Creazione gruppo con il primo utente come proprietario
                let group_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("bench_group_{}", g));
                let group_payload = json!({
                    "name": group_dto.name,
                    "description": group_dto.description
                });

                let group_response = client
                    .post(&format!("{}/api/group_chat/create", base_url))
                    .bearer_auth(&group_tokens[0])
                    .json(&group_payload)
                    .send()
                    .await
                    .expect("Failed to create group");

                if !group_response.status().is_success() {
                    panic!("Group creation failed for group {}: {}", g, group_response.status());
                }

                let group_data: serde_json::Value = group_response.json().await
                    .expect("Failed to parse group response");

                stats.groups_created.fetch_add(1, Ordering::SeqCst);

                let group_id = group_data["data"]["id"].as_i64().unwrap() as i32;

                // Invio inviti agli altri utenti del gruppo
                let mut invitation_tasks = Vec::new();
                for user_idx in 1..users_per_group {
                    let invitation_client = client.clone();
                    let base_url = base_url.clone();
                    let owner_token = group_tokens[0].clone();
                    let invitee_user_data = group_users[user_idx].clone();
                    let invitee_token = group_tokens[user_idx].clone();
                    let stats = stats.clone();
                    
                    invitation_tasks.push(tokio::spawn(async move {
                        let user_id = invitee_user_data["id"].as_i64().unwrap() as i32;
                        
                        // Invio invito
                        let invitation_payload = json!({
                            "to_user_id": user_id,
                            "group_chat_id": group_id,
                            "role_at_join": "member"
                        });

                        let invitation_response = invitation_client
                            .post(&format!("{}/api/invitation/send", base_url))
                            .bearer_auth(&owner_token)
                            .json(&invitation_payload)
                            .send()
                            .await
                            .expect("Failed to send invitation");

                        if !invitation_response.status().is_success() {
                            panic!("Invitation send failed: {}", invitation_response.status());
                        }

                        let invitation_data: serde_json::Value = invitation_response.json().await
                            .expect("Failed to parse invitation response");

                        stats.invitations_created.fetch_add(1, Ordering::SeqCst);

                        let invitation_id = invitation_data["data"]["id"].as_i64().unwrap() as i32;

                        // Accettazione invito
                        let accept_payload = json!({
                            "status": InvitationStatus::Accepted,
                            "invitation_id": invitation_id,
                        });

                        let accept_response = invitation_client
                            .patch(&format!("{}/api/invitation/update-status", base_url))
                            .bearer_auth(&invitee_token)
                            .json(&accept_payload)
                            .send()
                            .await
                            .expect("Failed to accept invitation");

                        if !accept_response.status().is_success() {
                            panic!("Invitation acceptance failed: {}", accept_response.status());
                        }

                        stats.update_invitation_status.fetch_add(1, Ordering::SeqCst);

                        (user_id, invitee_token)
                    }));
                }

                // Attesa completamento inviti
                let mut final_user_tokens = vec![group_tokens[0].clone()];
                for task in invitation_tasks {
                    let (_, token) = task.await.expect("Invitation task failed");
                    final_user_tokens.push(token);
                }

                (group_users, final_user_tokens, group_id)
            });
            
            group_creation_tasks.push(task);
        }

        // Attesa completamento di tutti i gruppi
        for task in group_creation_tasks {
            let (group_users, group_tokens, group_id) = task.await.expect("Group creation task failed");
            
            all_users.extend(group_users);
            all_tokens.extend(group_tokens);
            groups.push(group_id);
        }

        println!("✅ Creati {} utenti in {} gruppi tramite API alle {}", all_users.len(), NUM_GROUPS, Utc::now().format("%Y-%m-%d %H:%M:%S"));

        // Strutture dati condivise
        let stop_flag = Arc::new(AtomicBool::new(false));
        let created_messages: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));

        // --- Avvio task WebSocket users (devono connettersi prima) ---
        let mut websocket_handles = Vec::new();
        let mut user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group_id = groups[g];
            let ws_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP + PAGINATED_READERS_PER_GROUP;

            // WebSocket passive users (solo ricevono messaggi)
            for ws_user_idx in ws_start_idx..ws_start_idx + WEBSOCKET_PASSIVE_USERS {
                let token = all_tokens[ws_user_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let mut rng = rng.clone();
                let addr = server_host.clone();

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
            let addr = server_host.clone();
            for ws_user_idx in ws_start_idx + WEBSOCKET_PASSIVE_USERS..ws_start_idx + WEBSOCKET_USERS_PER_GROUP {
                let token = all_tokens[ws_user_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
                let mut rng = rng.clone();

                let handle = tokio::spawn({
                    let addr = addr.clone();
                    async move {
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
                }
                });

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                websocket_handles.push(handle);
            }

            user_idx += users_per_group;
        }

        // Pausa per permettere le connessioni WebSocket
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // --- Avvio task NOT READ READERS ---
        let mut not_read_handles = Vec::new();
        user_idx = 0;

        for g in 0..NUM_GROUPS {
            let group_id = groups[g];
            let readers_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP;

            for reader_idx in readers_start_idx..readers_start_idx + NOT_READ_READERS_PER_GROUP {
                let token = all_tokens[reader_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
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
                                                for message in messages.iter() {
                                                    if let Some(msg_id) = message.get("id").and_then(|id| id.as_i64()) {

                                                        let payload = json!({
                                                            "text_message_id": msg_id,
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
            let group_id = groups[g];
            let readers_start_idx = user_idx + MESSAGE_SENDERS_PER_GROUP + NOT_READ_READERS_PER_GROUP;

            for reader_idx in readers_start_idx..readers_start_idx + PAGINATED_READERS_PER_GROUP {
                let token = all_tokens[reader_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let base_url = base_url.clone();
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
            let group_id = groups[g];
            
            for sender_idx in user_idx..user_idx + MESSAGE_SENDERS_PER_GROUP {
                let token = all_tokens[sender_idx].clone();
                let stop_flag = Arc::clone(&stop_flag);
                let stats = Arc::clone(&stats);
                let created_messages = Arc::clone(&created_messages);
                let base_url = base_url.clone();
                let mut rng = rng.clone();                let handle = tokio::spawn(async move {
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

        // NOTE: CPU monitoring disabled when using Docker server
        // cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop CPU monitoring");

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

        // NOTE: No need to shutdown Docker server
        // let _ = shutdown.send(());

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
