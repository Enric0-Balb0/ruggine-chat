use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use tokio::sync::{oneshot, Semaphore};
use crate::common::{
    cleanup_all_cpu_usage_log, cleanup_group_chat, cleanup_user_by_email,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    add_test_user_to_a_group, start_test_server, cleanup_test_user_from_a_group_chat,
    cleanup_text_messages
};

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use serial_test::serial;
    use futures::stream::{self, StreamExt};
    use reqwest::Client;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use futures_util::stream::FuturesUnordered;
    use tokio::sync::Semaphore;
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
        const NUM_GROUPS: usize = 100;
        const USERS_PER_GROUP: usize = 10;
        const MSGS_PER_USER: usize = 100;

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

        for (token, payload) in messages.clone() {
            let client = Arc::clone(&client);
            let base_url = base_url.clone();
            let semaphore = Arc::clone(&semaphore);
            let progress_counter = Arc::clone(&progress_counter);

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
                        resp.json::<serde_json::Value>().await.ok()
                            .and_then(|j| j.get("data")?.get("id")?.as_i64())
                            .map(|id| id as i32)
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

        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop monitoring");

        println!("📊 RISULTATI BENCHMARK:");
        println!("  Totale tentativi: {}", messages.len());
        println!("  Successi: {}", successful_messages);
        println!("  Falliti: {}", messages.len() - successful_messages);
        println!("  Tempo creazione messaggi: {:?}", message_creation_duration);
        println!("  Msg/s: {:.2}", successful_messages as f64 / message_creation_duration.as_secs_f64());
        println!("  Tempo medio per messaggio: {:?}", message_creation_duration / successful_messages as u32);

        // Cleanup batch
        let created_message_ids: Vec<i32> = results.into_iter().flatten().collect();
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

        assert_eq!(successful_messages, messages.len(), "Success rate troppo basso");
    }
}
