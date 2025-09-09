use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use tokio::sync::{oneshot, Semaphore};
use chrono::Utc;
use crate::common::{
    cleanup_all_cpu_usage_log, cleanup_group_chat, cleanup_user_by_email,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    add_test_user_to_a_group, start_test_server, cleanup_test_user_from_a_group_chat,
    cleanup_text_messages, mark_message_as_sent, cleanup_text_message_info_by_message_id
};

#[cfg(test)]
mod benchmark_tests {
    use std::collections::HashMap;
    use super::*;
    use serial_test::serial;
    use futures::stream::{self, StreamExt};
    use reqwest::Client;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use futures_util::stream::FuturesUnordered;
    use tokio::sync::{Mutex, Semaphore};
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;
    use ruggine_server::config::parameter;
    use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{common, create_admin_login_and_get_token, get_database};

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }



    /// Benchmark test: N users creating M messages concurrently
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_concurrent_message_creation_benchmark_large() {
        // Cleanup iniziale
        cleanup_all_cpu_usage_log().await;

        let db = get_database().await;
        let mut cpu_usage_log_service = CpuUsageLogService::new(
            Arc::new(CpuUsageLogRepository::new(&db))
        );
        cpu_usage_log_service.set_monitoring_interval_ms(1000);
        cpu_usage_log_service.start_monitoring().await.expect("Failed to start CPU monitoring");

        // Parametri
        const NUM_GROUPS: usize = 50;
        const USERS_PER_GROUP: usize = 10;
        const MSGS_PER_USER: usize = 10;

        info!("🚀 Benchmark: {} gruppi × {} utenti × {} messaggi", NUM_GROUPS, USERS_PER_GROUP, MSGS_PER_USER);

        // Avvio server di test
        let (addr, shutdown) = start_test_server().await;
        let base_url = format!("http://{}", addr);

        let start_time = Instant::now();

        // --- Creazione utenti globali ---
        let mut users = Vec::with_capacity(NUM_GROUPS * USERS_PER_GROUP);
        let mut tokens = Vec::with_capacity(NUM_GROUPS * USERS_PER_GROUP);

        for i in 0..(NUM_GROUPS * USERS_PER_GROUP) {
            let (user, _, token) = create_login_and_get_token(format!("bench_user_{}", i)).await;
            users.push(user);
            tokens.push(token);
        }

        println!("✅ Creati {} utenti", users.len());

        // --- Creazione gruppi e membership ---
        let mut groups = Vec::with_capacity(NUM_GROUPS);
        for g in 0..NUM_GROUPS {
            let start_idx = g * USERS_PER_GROUP;

            let group = create_test_group_chat_with_invitation_and_membership(
                format!("bench_group_{}", g).as_str(),
                users[start_idx].id
            ).await;

            // Aggiungi altri utenti al gruppo
            for u in &users[start_idx + 1..start_idx + USERS_PER_GROUP] {
                add_test_user_to_a_group(u.id, &group).await;
            }

            groups.push(group);
        }

        println!("✅ Creati {} gruppi con membership", NUM_GROUPS);

        // --- Preparazione payload messaggi ---
        let mut messages = Vec::new();
        for (group_idx, group) in groups.iter().enumerate() {
            let start_idx = group_idx * USERS_PER_GROUP;
            for u_idx in start_idx..start_idx + USERS_PER_GROUP {
                for msg_idx in 0..MSGS_PER_USER {
                    messages.push((
                        tokens[u_idx].clone(),
                        json!({
                        "content": format!("Benchmark message {} from user {}", msg_idx, u_idx),
                        "group_chat_id": group.id
                        })
                    ));
                }
            }
        }

        println!("🔄 Invio di {} messaggi concorrenti...", messages.len());

        // --- Invio concorrente ---
        let client = Arc::new(Client::new());
        let semaphore = Arc::new(Semaphore::new(20)); // concorrenza maggiore
        let mut futures = FuturesUnordered::new();

        let total_messages = messages.len();
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let progress_step = total_messages / 100; // 1% step

        let group_to_message_ids: Arc<Mutex<HashMap<i32, Vec<i32>>>> = Arc::new(Mutex::new(HashMap::new()));

        for (token, payload) in messages.clone() {
            let client = Arc::clone(&client);
            let base_url = base_url.clone();
            let semaphore = Arc::clone(&semaphore);
            let progress_counter = Arc::clone(&progress_counter);
            let group_to_message_ids = Arc::clone(&group_to_message_ids);

            futures.push(async move {
                let permit = semaphore.acquire().await.unwrap();

                let fut = client
                    .post(&format!("{}/api/text_message/create", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send();

                let res = timeout(Duration::from_secs(10), fut).await;
                drop(permit);

                // Aggiorna contatore e stampa progresso
                let completed = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
                if completed % progress_step == 0 {
                    println!("📤 Progresso: {:.0}% ({}/{})", completed as f64 / total_messages as f64 * 100.0, completed, total_messages);
                }

                match res {
                    Ok(Ok(resp)) if resp.status().is_success() => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(msg_id) = json.get("data").and_then(|d| d.get("id"))?.as_i64() {
                                let msg_id = msg_id as i32;
                                // --- Aggiorna mappa group_id -> message_ids ---
                                if let Some(group_id) = payload.get("group_chat_id").and_then(|v| v.as_i64()) {
                                    let mut map = group_to_message_ids.lock().await;
                                    map.entry(group_id as i32).or_default().push(msg_id);
                                }
                                return Some(msg_id);
                            }
                        }
                        None
                    }
                    _ => None,
                }
            });
        }

        let mut results = Vec::new();
        while let Some(res) = futures.next().await {
            results.push(res);
        }

        let message_creation_duration = start_time.elapsed();
        let successful_messages: usize = results.iter().filter(|r| r.is_some()).count();

        println!("📊 RISULTATI BENCHMARK CREAZIONE:");
        println!("  Totale tentativi: {}", messages.len());
        println!("  Successi: {}", successful_messages);
        println!("  Falliti: {}", messages.len() - successful_messages);
        println!("  Tempo creazione messaggi: {:?}", message_creation_duration);
        println!("  Msg/s: {:.2}", successful_messages as f64 / message_creation_duration.as_secs_f64());
        println!("  Tempo medio per messaggio: {:?}", message_creation_duration / successful_messages as u32);

        // --- PARTE DI LETTURA DEI MESSAGGI ---
        println!("\n🔍 Avvio benchmark lettura messaggi...");

        // Ottieni gli ID dei messaggi creati con successo e mantieni la mappatura con i gruppi
        let created_message_ids: Vec<i32> = results.iter().flatten().cloned().collect();

        // println!("✅ Messaggi marcati come inviati per tutti gli utenti");

        // Pausa per permettere la propagazione
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Benchmark: Lettura concorrente dei messaggi non letti
        let reading_start = Instant::now();
        let semaphore_read = Arc::new(Semaphore::new(20));

        let mut reading_futures = FuturesUnordered::new();

        for (group_idx, group) in groups.iter().enumerate() {
            let start_idx = group_idx * USERS_PER_GROUP;
            for u_idx in start_idx..start_idx + USERS_PER_GROUP {
                let client = Arc::clone(&client);
                let base_url = base_url.clone();
                let token = tokens[u_idx].clone();
                let semaphore_read = Arc::clone(&semaphore_read);
                let group_id = group.id;

                reading_futures.push(async move {
                    let permit = semaphore_read.acquire().await.unwrap();

                    // Prima otteniamo i messaggi non letti
                    let get_fut = client
                        .get(&format!("{}/api/text_message/group/{}/messages/not-read-yet", base_url, group_id))
                        .header("Authorization", format!("Bearer {}", token))
                        .send();

                    let response_result = timeout(Duration::from_secs(10), get_fut).await;

                    drop(permit);

                    match response_result {
                        Ok(Ok(resp)) if resp.status().is_success() => {
                            match resp.json::<serde_json::Value>().await {
                                Ok(json) => {
                                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                                        let messages_count = data.len();
                                        Some((u_idx, messages_count, data.clone()))
                                    } else {
                                        eprintln!("❌ User {}: Nessun campo data nell'array", u_idx);
                                        None
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ User {}: Errore nel parsing JSON lettura: {}", u_idx, e);
                                    None
                                }
                            }
                        }
                        Ok(Ok(resp)) => {
                            eprintln!("❌ User {}: Lettura fallita con status: {}", u_idx, resp.status());
                            None
                        }
                        Ok(Err(e)) => {
                            eprintln!("❌ User {}: Errore nella richiesta lettura: {}", u_idx, e);
                            None
                        }
                        Err(_) => {
                            eprintln!("⏰ User {}: Timeout nella richiesta lettura", u_idx);
                            None
                        }
                    }
                });
            }
        }

        // Raccogliamo i risultati della lettura
        let mut read_results = Vec::new();
        let mut successful_reads = 0;
        let mut total_messages_read = 0;

        while let Some(result) = reading_futures.next().await {
            if let Some((user_idx, messages_count, _messages_data)) = result {
                read_results.push((user_idx, messages_count));
                successful_reads += 1;
                total_messages_read += messages_count;
            }
        }

        let reading_duration = reading_start.elapsed();

        // Benchmark: Marcatura concorrente come "letti" - solo messaggi del proprio gruppo
        let mark_read_start = Instant::now();
        let semaphore_mark = Arc::new(Semaphore::new(20));

        let mut mark_read_futures = FuturesUnordered::new();

        // Per ogni gruppo, gli utenti marcano solo i messaggi del loro gruppo
        for (group_idx, group) in groups.iter().enumerate() {
            let start_idx = group_idx * USERS_PER_GROUP;

            // Trova i messaggi che appartengono a questo gruppo
            let group_messages = {
                let map = group_to_message_ids.lock().await;
                map.get(&group.id).cloned().unwrap_or_default()
            };

            // Per ogni utente del gruppo
            for u_idx in start_idx..start_idx + USERS_PER_GROUP {
                // Per ogni messaggio del gruppo
                for message_id in &group_messages {
                    let client = Arc::clone(&client);
                    let base_url = base_url.clone();
                    let token = tokens[u_idx].clone();
                    let semaphore_mark = Arc::clone(&semaphore_mark);
                    let message_id = *message_id;

                    mark_read_futures.push(async move {
                        let permit = semaphore_mark.acquire().await.unwrap();

                        let mark_read_payload = json!({
                            "text_message_id": message_id,
                        });

                        let fut = client
                            .patch(&format!("{}/api/text_message/update_read_at", base_url))
                            .header("Authorization", format!("Bearer {}", token))
                            .header("Content-Type", "application/json")
                            .json(&mark_read_payload)
                            .send();

                        let response_result = timeout(Duration::from_secs(5), fut).await;

                        drop(permit);

                        match response_result {
                            Ok(Ok(resp)) if resp.status().is_success() => {
                                Some((u_idx, message_id))
                            }
                            Ok(Ok(resp)) => {
                                eprintln!("❌ User {}: Mark read fallito per messaggio {} con status: {}", u_idx, message_id, resp.status());
                                None
                            }
                            Ok(Err(e)) => {
                                eprintln!("❌ User {}: Errore nella richiesta mark read per messaggio {}: {}", u_idx, message_id, e);
                                None
                            }
                            Err(_) => {
                                eprintln!("⏰ User {}: Timeout nella richiesta mark read per messaggio {}", u_idx, message_id);
                                None
                            }
                        }
                    });
                }
            }
        }

        // Raccogliamo i risultati del mark as read
        let mut marked_read = Vec::new();
        let mut successful_mark_reads = 0;

        while let Some(result) = mark_read_futures.next().await {
            if let Some((user_idx, message_id)) = result {
                marked_read.push((user_idx, message_id));
                successful_mark_reads += 1;
            }
        }

        let mark_read_duration = mark_read_start.elapsed();

        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop CPU monitoring");

        // Statistiche finali complete
        // Ogni gruppo ha USERS_PER_GROUP utenti e ogni utente del gruppo può leggere tutti i messaggi creati per quel gruppo
        // Ogni gruppo ha USERS_PER_GROUP * MSGS_PER_USER messaggi
        let messages_per_group = USERS_PER_GROUP * MSGS_PER_USER;
        let expected_mark_reads = NUM_GROUPS * USERS_PER_GROUP * messages_per_group;
        let total_users = NUM_GROUPS * USERS_PER_GROUP;

        println!("\n📊 RISULTATI BENCHMARK COMPLETO:");
        println!("=== CREAZIONE MESSAGGI ===");
        println!("  Totale tentativi: {}", messages.len());
        println!("  Successi: {}", successful_messages);
        println!("  Falliti: {}", messages.len() - successful_messages);
        println!("  Tempo creazione messaggi: {:?}", message_creation_duration);
        println!("  Msg/s: {:.2}", successful_messages as f64 / message_creation_duration.as_secs_f64());
        println!("  Tempo medio per messaggio: {:?}", message_creation_duration / successful_messages as u32);

        println!("\n=== LETTURA MESSAGGI ===");
        println!("  Utenti lettori: {}", total_users);
        println!("  Messaggi totali creati: {}", successful_messages);
        println!("  Letture riuscite: {}/{}", successful_reads, total_users);
        println!("  Tempo lettura messaggi: {:?}", reading_duration);
        println!("  Tempo medio per lettura: {:?}", reading_duration / successful_reads.max(1));
        println!("  **Tempo medio per lettura singolo messaggio**: {:?}", Duration::from_nanos(reading_duration.as_nanos() as u64 / total_messages_read as u64));
        println!("  Letture/s: {:.2}", successful_reads as f64 / reading_duration.as_secs_f64());
        println!("  Messaggi totali letti: {}", total_messages_read);
        println!("  Media messaggi per utente: {:.2}", total_messages_read as f64 / successful_reads.max(1) as f64);

        println!("\n=== MARCATURA COME LETTI ===");
        println!("  Mark as read riuscite: {}/{}", successful_mark_reads, expected_mark_reads);
        println!("  Tempo mark as read: {:?}", mark_read_duration);
        println!("  Tempo medio per mark as read: {:?}", mark_read_duration / successful_mark_reads.max(1));
        println!("  Mark reads/s: {:.2}", successful_mark_reads as f64 / mark_read_duration.as_secs_f64());
        println!("  Success rate mark as read: {:.2}%", (successful_mark_reads as f64 / expected_mark_reads as f64) * 100.0);

        // Cleanup delle info dei messaggi (read_at, sent_at)
        for message_id in &created_message_ids {
            cleanup_text_message_info_by_message_id(*message_id).await;
        }

        // Cleanup batch
        if !created_message_ids.is_empty() {
            cleanup_text_messages(created_message_ids).await;
        }

        for g in 0..NUM_GROUPS {
            let start_idx = g * USERS_PER_GROUP;
            let group = &groups[g];
            for u in &users[start_idx..start_idx + USERS_PER_GROUP] {
                cleanup_test_user_from_a_group_chat(u.id, group.id).await;
            }
            cleanup_group_chat(group.id).await;
        }

        for user in &users {
            cleanup_user_by_email(user.email.clone()).await;
        }

        let _ = shutdown.send(());

        // Verifiche finali
        let read_success_rate = successful_reads as f64 / total_users as f64;
        let mark_read_success_rate = successful_mark_reads as f64 / expected_mark_reads as f64;

        assert_eq!(successful_messages, messages.len(), "Success rate creazione messaggi troppo basso");
        assert!(read_success_rate >= 0.95, "Success rate lettura troppo basso: {:.2}%", read_success_rate * 100.0);
        assert!(mark_read_success_rate >= 0.95, "Success rate mark-as-read troppo basso: {:.2}%", mark_read_success_rate * 100.0);
    }
}
