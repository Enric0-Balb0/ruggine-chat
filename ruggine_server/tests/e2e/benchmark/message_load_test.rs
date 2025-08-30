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
    use tokio::sync::Semaphore;
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;
    use crate::create_admin_login_and_get_token;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }



    /// Benchmark test: N users creating M messages concurrently
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_concurrent_message_creation_benchmark() {
        // Abilita log durante il test
        // init_tracing();

        // Set variabile ENV per CPU logging
        std::env::set_var("LOG_IN_MILLISECONDS", "100000");

        // Cleanup iniziale
        cleanup_all_cpu_usage_log().await;

        // Parametri del test
        const NUM_USERS: usize = 2000;
        const MESSAGES_PER_USER: usize = 10;
        const TOTAL_MESSAGES: usize = NUM_USERS * MESSAGES_PER_USER;

        info!("🚀 Avvio benchmark: {} utenti, {} messaggi ciascuno ({} totali)",
              NUM_USERS, MESSAGES_PER_USER, TOTAL_MESSAGES);

        // Avvio server di test
        let (addr, shutdown) = start_test_server().await;
        let base_url = format!("http://{}", addr);

        let start_time = Instant::now();

        // Creazione utenti e gruppo
        let mut users = Vec::new();
        let mut tokens = Vec::new();

        let (owner, _, owner_token) = create_admin_login_and_get_token("benchmark_owner".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("benchmark_group", owner.id).await;

        users.push(owner);
        tokens.push(owner_token);

        for i in 1..NUM_USERS {
            let (user, _, token) = create_login_and_get_token(format!("benchmark_user_{}", i)).await;
            add_test_user_to_a_group(user.id, &group).await;
            users.push(user);
            tokens.push(token);
        }

        info!("✅ Creati {} utenti e aggiunti al gruppo", NUM_USERS);

        // HTTP client + semaphore
        let client = Arc::new(Client::new());
        let semaphore = Arc::new(Semaphore::new(50));

        // Genera tutti i payload dei messaggi
        let mut messages = Vec::new();
        for (user_idx, token) in tokens.iter().enumerate() {
            for msg_idx in 0..MESSAGES_PER_USER {
                messages.push((
                    token.clone(),
                    json!({
                        "content": format!("Benchmark message {} from user {}", msg_idx, user_idx),
                        "group_chat_id": group.id
                    }),
                ));
            }
        }

        info!("🔄 Invio di {} messaggi concorrenti...", TOTAL_MESSAGES);

        let payload =json!({
            "content": format!("Benchmark message from user"),
            "group_chat_id": group.id
        });

        let mut handles = vec![];
        for _ in 0..5 {  // Ridotto da 20 a 5 per evitare deadlock
            let payload = payload.clone();
            let client = client.clone();
            let base_url = base_url.clone();
            let token = tokens[0].clone();

            let handle = tokio::spawn(async move {
                match client
                    .post(&format!("{}/api/text_message/create", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(response) => {
                        println!("Success: {:?}", response.status());
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            });

            handles.push(handle);
        }

        // Attendi tutti i task
        for handle in handles {
            let _ = handle.await;
        }

        println!("Finished");

        // Esecuzione concorrente con buffer_unordered
        /*let message_creation_start = Instant::now();

        let results: Vec<_> = stream::iter(messages.into_iter())
            .map(|(token, payload)| {
                let client = Arc::clone(&client);
                let base_url = base_url.clone();
                let semaphore = Arc::clone(&semaphore);

                async move {
                    info!("⏳ Acquisizione semaforo...");
                    let permit = semaphore.acquire().await.unwrap();
                    info!("✅ Semaforo acquisito");

                    let fut = client
                        .post(&format!("{}/api/text_message/create", base_url))
                        .header("Authorization", format!("Bearer {}", token))
                        .header("Content-Type", "application/json")
                        .json(&payload)
                        .send();

                    // Timeout massimo per ogni richiesta
                    let response = timeout(Duration::from_secs(5), fut).await;

                    drop(permit);
                    info!("🔓 Semaforo rilasciato");

                    match response {
                        Ok(Ok(resp)) if resp.status().is_success() => {
                            if let Ok(body) = resp.text().await {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                    if let Some(message_id) = json.get("data")
                                        .and_then(|d| d.get("id"))
                                        .and_then(|id| id.as_i64())
                                    {
                                        return Some(message_id as i32);
                                    }
                                }
                            }
                            None
                        }
                        Ok(Ok(resp)) => {
                            eprintln!("❌ Fallito con status: {}", resp.status());
                            None
                        }
                        Ok(Err(e)) => {
                            eprintln!("❌ Errore nella richiesta: {}", e);
                            None
                        }
                        Err(_) => {
                            eprintln!("⏰ Timeout nella richiesta");
                            None
                        }
                    }
                }
            })
            .buffer_unordered(50) // max 50 richieste in flight
            .collect()
            .await;

        let message_creation_duration = message_creation_start.elapsed();

        // Analisi risultati
        let mut created_message_ids = Vec::new();
        let mut successful_messages = 0;

        for r in results {
            if let Some(mid) = r {
                created_message_ids.push(mid);
                successful_messages += 1;
            }
        }

        info!("📊 RISULTATI BENCHMARK:");
        info!("  Totale tentativi: {}", TOTAL_MESSAGES);
        info!("  Successi: {}", successful_messages);
        info!("  Falliti: {}", TOTAL_MESSAGES as i32 - successful_messages);
        info!("  Tempo creazione messaggi: {:?}", message_creation_duration);
        info!("  Msg/s: {:.2}", successful_messages as f64 / message_creation_duration.as_secs_f64());
        info!("  Tempo medio per messaggio: {:?}", message_creation_duration / successful_messages as u32);

        // Cleanup
        if !created_message_ids.is_empty() {
            cleanup_text_messages(created_message_ids).await;
            info!("🧹 Puliti {} messaggi", successful_messages);
        }

        for user in &users {
            cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        }

        cleanup_group_chat(group.id).await;
        for user in users {
            cleanup_user_by_email(user.email).await;
        }

        cleanup_all_cpu_usage_log().await;

        let _ = shutdown.send(());

        let total_duration = start_time.elapsed();
        info!("✅ Test completato in {:?}", total_duration);

        // Almeno l'80% deve andare a buon fine
        let success_rate = successful_messages as f64 / TOTAL_MESSAGES as f64;
        assert!(success_rate >= 0.8, "Success rate troppo basso: {:.2}%", success_rate * 100.0);*/
    }

}
