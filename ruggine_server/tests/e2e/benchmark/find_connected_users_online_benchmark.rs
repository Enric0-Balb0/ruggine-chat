use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use reqwest::Client;
use serde_json::json;
use tokio::sync::{oneshot, Semaphore};
use crate::common::{
    cleanup_all_cpu_usage_log, cleanup_group_chat, cleanup_user_by_email,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    add_test_user_to_a_group, start_test_server, cleanup_test_user_from_a_group_chat,
    get_database
};

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use serial_test::serial;
    use futures::stream::StreamExt;
    use reqwest::Client;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use futures_util::stream::FuturesUnordered;
    use tokio::sync::Semaphore;
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;
    use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    use ruggine_server::service::user_service::{UserService, UserServiceTrait};
    use ruggine_server::factory::user_factory::UserFactory;
    use ruggine_server::utils::service_initializer::ServiceInitializer;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }

    /// Benchmark test: N users finding connected users online concurrently
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_concurrent_find_connected_users_online() {
        // Cleanup iniziale
        cleanup_all_cpu_usage_log().await;

        let db = get_database().await;
        let mut cpu_usage_log_service = CpuUsageLogService::new(
            Arc::new(CpuUsageLogRepository::new(&db))
        );
        cpu_usage_log_service.set_monitoring_interval_ms(1000);
        cpu_usage_log_service.start_monitoring().await.expect("Failed to start CPU monitoring");

        // Parametri
        const NUM_GROUPS: usize = 1000;
        const USERS_PER_GROUP: usize = 10;

        info!("🚀 Benchmark: {} gruppi × {} utenti per find_connected_users_online", NUM_GROUPS, USERS_PER_GROUP);

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

        // --- Imposta tutti gli utenti come online ---
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let update_online_dto = UserFactory::fake_update_online_dto_true();

        for user in &users {
            user_service.update_online(user.id, update_online_dto.clone()).await.unwrap();
        }

        println!("✅ Impostati tutti gli utenti come online");

        let setup_duration = start_time.elapsed();
        println!("⏱️ Tempo setup: {:?}", setup_duration);

        // --- BENCHMARK: Lettura concorrente degli utenti connessi online ---
        println!("\n🔍 Avvio benchmark find_connected_users_online...");

        let reading_start = Instant::now();
        let semaphore_read = Arc::new(Semaphore::new(20));
        let mut reading_futures = FuturesUnordered::new();

        let total_users = NUM_GROUPS * USERS_PER_GROUP;
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let progress_step = total_users / 100; // 1% step

        for (user_idx, token) in tokens.iter().enumerate() {
            let client = Arc::new(Client::new());
            let base_url = base_url.clone();
            let token = token.clone();
            let semaphore_read = Arc::clone(&semaphore_read);
            let progress_counter = Arc::clone(&progress_counter);

            reading_futures.push(async move {
                let permit = semaphore_read.acquire().await.unwrap();

                let get_fut = client
                    .get(&format!("{}/api/group_membership/connected_users_online", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .send();

                let response_result = timeout(Duration::from_secs(10), get_fut).await;

                drop(permit);

                // Aggiorna contatore e stampa progresso
                let completed = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
                if completed % progress_step == 0 {
                    println!("📤 Progresso: {:.0}% ({}/{})", completed as f64 / total_users as f64 * 100.0, completed, total_users);
                }

                match response_result {
                    Ok(Ok(resp)) if resp.status().is_success() => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                                    let connected_users_count = data.len();
                                    Some((user_idx, connected_users_count))
                                } else {
                                    eprintln!("❌ User {}: Nessun campo data nell'array", user_idx);
                                    None
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ User {}: Errore nel parsing JSON lettura: {}", user_idx, e);
                                None
                            }
                        }
                    }
                    Ok(Ok(resp)) => {
                        eprintln!("❌ User {}: Lettura fallita con status: {}", user_idx, resp.status());
                        None
                    }
                    Ok(Err(e)) => {
                        eprintln!("❌ User {}: Errore nella richiesta lettura: {}", user_idx, e);
                        None
                    }
                    Err(_) => {
                        eprintln!("⏰ User {}: Timeout nella richiesta lettura", user_idx);
                        None
                    }
                }
            });
        }

        // Raccogliamo i risultati della lettura
        let mut read_results = Vec::new();
        let mut successful_reads = 0;
        let mut total_connected_users_found = 0;

        while let Some(result) = reading_futures.next().await {
            if let Some((user_idx, connected_users_count)) = result {
                read_results.push((user_idx, connected_users_count));
                successful_reads += 1;
                total_connected_users_found += connected_users_count;
            }
        }

        let reading_duration = reading_start.elapsed();

        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop CPU monitoring");

        // Statistiche finali complete
        let expected_connected_per_user = USERS_PER_GROUP - 1; // ogni utente dovrebbe vedere gli altri del suo gruppo (escluso se stesso)

        println!("\n📊 RISULTATI BENCHMARK FIND_CONNECTED_USERS_ONLINE:");
        println!("=== SETUP ===");
        println!("  Gruppi creati: {}", NUM_GROUPS);
        println!("  Utenti per gruppo: {}", USERS_PER_GROUP);
        println!("  Utenti totali: {}", total_users);
        println!("  Tempo setup: {:?}", setup_duration);
        
        println!("\n=== RICERCA UTENTI CONNESSI ===");
        println!("  Richieste totali: {}", total_users);
        println!("  Richieste riuscite: {}/{}", successful_reads, total_users);
        println!("  Tempo ricerca: {:?}", reading_duration);
        println!("  Tempo medio per richiesta: {:?}", reading_duration / successful_reads.max(1));
        println!("  Richieste/s: {:.2}", successful_reads as f64 / reading_duration.as_secs_f64());
        println!("  Utenti connessi totali trovati: {}", total_connected_users_found);
        println!("  Media utenti connessi per utente: {:.2}", total_connected_users_found as f64 / successful_reads.max(1) as f64);
        println!("  Utenti connessi attesi per utente: {}", expected_connected_per_user);

        // --- Cleanup ---
        println!("\n🧹 Avvio cleanup...");

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

        assert!(read_success_rate >= 0.95, "Success rate lettura troppo basso: {:.2}%", read_success_rate * 100.0);
        
        // Verifica che ogni utente trovi circa il numero atteso di utenti connessi
        let avg_connected_per_user = total_connected_users_found as f64 / successful_reads.max(1) as f64;
        let expected_range = (expected_connected_per_user as f64 * 0.8)..(expected_connected_per_user as f64 * 1.2);
        assert!(expected_range.contains(&avg_connected_per_user), 
            "Media utenti connessi per utente fuori dal range atteso: {:.2} (atteso: {})", 
            avg_connected_per_user, expected_connected_per_user);

        println!("✅ Test completato con successo!");
    }
}
