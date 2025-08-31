use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Semaphore;
use crate::common::{
    cleanup_all_cpu_usage_log, cleanup_group_chat, cleanup_user_by_email,
    create_login_and_get_token, start_test_server, cleanup_invitation,
    cleanup_group_membership_by_invitation_id, get_database,
    create_admin_login_and_get_token, create_test_user
};

#[cfg(test)]
mod invitation_benchmark_tests {
    use super::*;
    use serial_test::serial;
    use futures_util::stream::FuturesUnordered;
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;
    use ruggine_server::config::parameter;
    use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use ruggine_server::factory::group_chat_factory::GroupChatFactory;
    use futures_util::StreamExt;
    use crate::cleanup_test_user_from_a_group_chat;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }

    /// Benchmark test: Admin crea un gruppo, manda N inviti e N utenti li accettano concorrentemente
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_concurrent_invitation_acceptance_benchmark_multiple_group_admins() {
        // init_tracing(); // Abilita log durante il test

        // Cleanup iniziale
        cleanup_all_cpu_usage_log().await;

        let db = get_database().await;
        let mut cpu_usage_log_service = CpuUsageLogService::new(
            Arc::new(CpuUsageLogRepository::new(&db))
        );
        cpu_usage_log_service.set_monitoring_interval_ms(1000u64);
        cpu_usage_log_service.start_monitoring().await.expect("Failed to start monitoring CPU usage");

        // Parametri
        const TOTAL_GROUPS: usize = 10000;
        const USERS_PER_GROUP: usize = 10;
        const N_GROUPS_PER_ADMIN: usize = 10;

        println!("🚀 Avvio benchmark: {} gruppi, {} utenti per gruppo", TOTAL_GROUPS, USERS_PER_GROUP);

        // Avvio server
        let (addr, shutdown) = start_test_server().await;
        let base_url = format!("http://{}", addr);
        let start_time = Instant::now();
        let client = Arc::new(Client::new());

        // Creazione admin e gruppi
        let num_admins = TOTAL_GROUPS / N_GROUPS_PER_ADMIN;
        let mut admins = Vec::new();
        let mut admin_tokens = Vec::new();
        let mut groups = Vec::new();
        let mut group_creation_times = Vec::new();

        for i in 0..num_admins {
            let (admin_user, _, admin_token) = create_admin_login_and_get_token(format!("group_admin_{}", i)).await;
            admins.push(admin_user);
            admin_tokens.push(admin_token.clone());

            // Creazione N_GROUPS_PER_ADMIN gruppi per ogni admin
            for j in 0..N_GROUPS_PER_ADMIN {
                let group_start = Instant::now();
                let idx = i * N_GROUPS_PER_ADMIN + j;
                let group_create_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("group_{}", idx));
                let payload = json!({
                "name": group_create_dto.name,
                "description": group_create_dto.description
            });

                let resp = client
                    .post(&format!("{}/api/group_chat/create", base_url))
                    .header("Authorization", format!("Bearer {}", admin_token))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                    .expect("Failed to create group");

                assert!(resp.status().is_success(), "Group creation failed");

                let data: serde_json::Value = resp.json().await.expect("Failed to parse group response");
                let group_id = data["data"]["id"].as_i64().unwrap() as i32;

                groups.push(group_id);
                group_creation_times.push(group_start.elapsed());
                println!("✅ Gruppo {} creato in {:?}", idx, group_creation_times[idx]);
            }
        }

        // Creazione utenti
        let mut users = Vec::new();
        let mut user_tokens = Vec::new();

        for i in 0..(TOTAL_GROUPS * USERS_PER_GROUP) {
            let (user, _, token) = create_login_and_get_token(format!("invite_user_{}", i)).await;
            users.push(user);
            user_tokens.push(token);
        }

        println!("✅ Creati {} utenti", TOTAL_GROUPS * USERS_PER_GROUP);

        // Invio inviti
        let invitation_start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(20));
        let mut invitation_futures = FuturesUnordered::new();
        let mut user_idx = 0;
        let mut user_invitation_mapping: Vec<Vec<Option<i32>>> = vec![vec![None; USERS_PER_GROUP]; TOTAL_GROUPS];

        for (group_idx, &group_id) in groups.iter().enumerate() {
            let admin_idx = group_idx / N_GROUPS_PER_ADMIN;
            let admin_token = admin_tokens[admin_idx].clone();

            for _ in 0..USERS_PER_GROUP {
                let client = Arc::clone(&client);
                let base_url = base_url.clone();
                let semaphore = Arc::clone(&semaphore);
                let user_id = users[user_idx].id;
                let current_idx = user_idx;
                let current_group_idx = group_idx;
                let admin_token = admin_token.clone();

                invitation_futures.push(async move {
                    let permit = semaphore.acquire().await.unwrap();
                        let payload = json!({
                        "to_user_id": user_id,
                        "group_chat_id": group_id,
                        "role_at_join": "member"
                    });

                    let fut = client
                        .post(&format!("{}/api/invitation/send", base_url))
                        .header("Authorization", format!("Bearer {}", admin_token))
                        .header("Content-Type", "application/json")
                        .json(&payload)
                        .send();

                    let result = timeout(Duration::from_secs(5), fut).await;
                    drop(permit);

                    let invitation_id = match result {
                        Ok(Ok(resp)) if resp.status().is_success() => {
                            match resp.json::<serde_json::Value>().await {
                                Ok(json) => json.get("data").and_then(|d| d.get("id")).and_then(|id| id.as_i64()).map(|id| id as i32),
                                Err(e) => { eprintln!("❌ Errore parsing JSON invito: {}", e); None }
                            }
                        }
                        Ok(Ok(resp)) => { eprintln!("❌ Invito fallito con status: {}", resp.status()); None }
                        Ok(Err(e)) => { eprintln!("❌ Errore richiesta invito: {}", e); None }
                        Err(_) => { eprintln!("⏰ Timeout richiesta invito"); None }
                    };

                    (current_group_idx, current_idx % USERS_PER_GROUP, invitation_id)
                });

                user_idx += 1;
            }
        }

        // Raccolta risultati inviti
        let mut successful_invitations = 0;
        while let Some((group_idx, user_in_group_idx, invitation_id)) = invitation_futures.next().await {
            user_invitation_mapping[group_idx][user_in_group_idx] = invitation_id;
            if invitation_id.is_some() {
                successful_invitations += 1;
            }
        }

        let invitation_duration = invitation_start.elapsed();
        println!("✅ Inviati {}/{} inviti in {:?}", successful_invitations, TOTAL_GROUPS * USERS_PER_GROUP, invitation_duration);

        // Accettazione inviti concorrente
        let acceptance_start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(20));
        let mut acceptance_futures = FuturesUnordered::new();

        for (group_idx, invitations) in user_invitation_mapping.iter().enumerate() {
            for (user_in_group_idx, &invitation_id_opt) in invitations.iter().enumerate() {
                if let Some(invitation_id) = invitation_id_opt {
                    let client = Arc::clone(&client);
                    let base_url = base_url.clone();
                    let user_token = user_tokens[group_idx * USERS_PER_GROUP + user_in_group_idx].clone();
                    let semaphore = Arc::clone(&semaphore);

                    acceptance_futures.push(async move {
                        let permit = semaphore.acquire().await.unwrap();
                        let payload = json!({
                        "invitation_id": invitation_id,
                        "status": "accepted"
                    });

                        let fut = client
                            .patch(&format!("{}/api/invitation/update-status", base_url))
                            .header("Authorization", format!("Bearer {}", user_token))
                            .header("Content-Type", "application/json")
                            .json(&payload)
                            .send();

                        let result = timeout(Duration::from_secs(5), fut).await;
                        drop(permit);

                        match result {
                            Ok(Ok(resp)) if resp.status().is_success() => {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(json) => {
                                        if json.get("data").and_then(|d| d.get("status")).and_then(|s| s.as_str()) == Some("accepted") {
                                            Some(invitation_id)
                                        } else { None }
                                    }
                                    Err(e) => { eprintln!("❌ Errore parsing JSON accettazione: {}", e); None }
                                }
                            }
                            Ok(Ok(resp)) => { eprintln!("❌ Accettazione fallita con status: {}", resp.status()); None }
                            Ok(Err(e)) => { eprintln!("❌ Errore richiesta accettazione: {}", e); None }
                            Err(_) => { eprintln!("⏰ Timeout richiesta accettazione"); None }
                        }
                    });
                }
            }
        }

        let mut accepted_invitations = Vec::new();
        let mut successful_acceptances = 0;
        while let Some(result) = acceptance_futures.next().await {
            if let Some(invitation_id) = result {
                accepted_invitations.push(invitation_id);
                successful_acceptances += 1;
            }
        }

        let acceptance_duration = acceptance_start.elapsed();
        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop monitoring");

        // Risultati
        println!("📊 RISULTATI BENCHMARK INVITI:");
        println!("  Gruppi totali: {}", TOTAL_GROUPS);
        println!("  Gruppi per admin: {}", N_GROUPS_PER_ADMIN);
        println!("  Utenti per gruppo: {}", USERS_PER_GROUP);
        println!("  Utenti totali: {}", TOTAL_GROUPS * USERS_PER_GROUP);
        println!("  Tempo invio inviti: {:?}", invitation_duration);
        println!("  Inviti inviati: {}/{}", successful_invitations, TOTAL_GROUPS * USERS_PER_GROUP);
        println!("  Tempo medio per invio invito: {:?}", invitation_duration / successful_invitations);
        println!("  Inviti/s: {:.2}", successful_invitations as f64 / invitation_duration.as_secs_f64());
        println!("  Accettazioni riuscite: {}/{}", successful_acceptances, successful_invitations);
        println!("  Tempo accettazione: {:?}", acceptance_duration);
        println!("  Tempo medio per accettazione: {:?}", acceptance_duration / successful_acceptances);
        println!("  Accettazioni/s: {:.2}", successful_acceptances as f64 / acceptance_duration.as_secs_f64());

        // Cleanup
        for invitation_id in accepted_invitations {
            cleanup_group_membership_by_invitation_id(invitation_id).await;
        }

        for invitations in user_invitation_mapping.iter() {
            for &invitation_id_opt in invitations {
                if let Some(invitation_id) = invitation_id_opt {
                    cleanup_invitation(invitation_id).await;
                }
            }
        }

        for (admin, _) in admins.iter().zip(admin_tokens.iter()) {
            for &group_id in &groups {
                cleanup_test_user_from_a_group_chat(admin.id, group_id).await;
            }
        }
        for &group_id in &groups {
            cleanup_group_chat(group_id).await;
        }

        for admin in admins {
            cleanup_user_by_email(admin.email).await;
        }
        for user in users {
            cleanup_user_by_email(user.email).await;
        }

        let _ = shutdown.send(());

        let total_duration = start_time.elapsed();
        println!("✅ Test completato in {:?}", total_duration);

        // Verifiche finali
        let invitation_success_rate = successful_invitations as f64 / (TOTAL_GROUPS * USERS_PER_GROUP) as f64;
        let acceptance_success_rate = successful_acceptances as f64 / successful_invitations as f64;

        assert!(invitation_success_rate >= 0.95, "Success rate inviti troppo basso: {:.2}%", invitation_success_rate * 100.0);
        assert!(acceptance_success_rate >= 0.95, "Success rate accettazioni troppo basso: {:.2}%", acceptance_success_rate * 100.0);
    }
}
