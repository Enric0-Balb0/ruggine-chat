use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::Client;
use serde_json::json;
use tokio::sync::Semaphore;
use chrono::Utc;
use crate::common::{
    cleanup_all_cpu_usage_log, cleanup_group_chat, cleanup_user_by_email,
    create_login_and_get_token, start_test_server, get_database,
    create_admin_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    add_test_user_to_a_group, cleanup_test_user_from_a_group_chat,
    create_test_text_messages_for_group, cleanup_text_messages,
    mark_message_as_sent, cleanup_text_message_info_by_message_id
};

#[cfg(test)]
mod message_read_benchmark_tests {
    use super::*;
    use serial_test::serial;
    use futures_util::stream::FuturesUnordered;
    use tokio::time::{timeout, Duration, Instant};
    use tracing::info;
    use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
    use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use futures_util::StreamExt;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    }

    /// Benchmark test: N utenti leggono concorrentemente 100 messaggi non letti di un gruppo
    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_concurrent_message_reading_benchmark() {
        // Abilita log durante il test
        // init_tracing();

        // Cleanup iniziale
        cleanup_all_cpu_usage_log().await;

        let db = get_database().await;
        let mut cpu_usage_log_service = CpuUsageLogService::new(
            Arc::new(CpuUsageLogRepository::new(&db))
        );
        cpu_usage_log_service.set_monitoring_interval_ms(1000u64);
        cpu_usage_log_service.start_monitoring().await.expect("Something went wrong starting monitoring cpu usage log");

        // Parametri del test
        const NUM_USERS: usize = 100;
        const NUM_MESSAGES: usize = 200;
        
        info!("🚀 Avvio benchmark lettura messaggi: {} utenti leggeranno {} messaggi", NUM_USERS, NUM_MESSAGES);

        // Avvio server di test
        let (addr, shutdown) = start_test_server().await;
        let base_url = format!("http://{}", addr);

        let start_time = Instant::now();

        // Creazione admin e gruppo
        let (admin_user, _, admin_token) = create_admin_login_and_get_token("message_read_admin".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("message_read_group", admin_user.id).await;
        
        info!("✅ Gruppo creato");

        // Creazione utenti reader
        let mut reader_users = Vec::new();
        let mut reader_tokens = Vec::new();
        
        for i in 0..NUM_USERS {
            let (user, _, token) = create_login_and_get_token(format!("message_reader_{}", i)).await;
            // Aggiungi utente al gruppo
            add_test_user_to_a_group(user.id, &group).await;
            reader_users.push(user);
            reader_tokens.push(token);
        }
        
        info!("✅ Creati {} utenti lettori e aggiunti al gruppo", NUM_USERS);

        // Creazione messaggi nel gruppo (l'admin manda i messaggi)
        let message_creation_start = Instant::now();
        let messages = create_test_text_messages_for_group(group.id, admin_user.id, NUM_MESSAGES).await;
        let message_creation_duration = message_creation_start.elapsed();
        
        info!("✅ Creati {} messaggi in {:?}", NUM_MESSAGES, message_creation_duration);

        // Marca tutti i messaggi come "sent" per tutti gli utenti (così sono visibili come "not read yet")
        let sent_time = Utc::now();
        for user in &reader_users {
            for message in &messages {
                mark_message_as_sent(user.id, message.id, sent_time).await;
            }
        }
        
        info!("✅ Messaggi marcati come inviati per tutti gli utenti");

        // Pausa per permettere la propagazione
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Benchmark: Lettura concorrente dei messaggi non letti
        let reading_start = Instant::now();
        let client = Arc::new(Client::new());
        let semaphore = Arc::new(Semaphore::new(20));
        
        let mut reading_futures = FuturesUnordered::new();
        
        for (user_idx, token) in reader_tokens.iter().enumerate() {
            let client = Arc::clone(&client);
            let base_url = base_url.clone();
            let token = token.clone();
            let semaphore = Arc::clone(&semaphore);
            let group_id = group.id;
            
            reading_futures.push(async move {
                let permit = semaphore.acquire().await.unwrap();
                
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
                                    Some((user_idx, messages_count, data.clone()))
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
        let mut total_messages_read = 0;
        
        while let Some(result) = reading_futures.next().await {
            if let Some((user_idx, messages_count, _messages_data)) = result {
                read_results.push((user_idx, messages_count));
                successful_reads += 1;
                total_messages_read += messages_count;
            }
        }

        let reading_duration = reading_start.elapsed();

        // Benchmark: Marcatura concorrente come "letti" (TUTTI i messaggi per TUTTI gli utenti)
        let mark_read_start = Instant::now();
        let semaphore = Arc::new(Semaphore::new(20));
        
        let mut mark_read_futures = FuturesUnordered::new();
        
        for (user_idx, token) in reader_tokens.iter().enumerate() { // Tutti gli utenti, non solo 10
            let client = Arc::clone(&client);
            let base_url = base_url.clone();
            let token = token.clone();
            let semaphore = Arc::clone(&semaphore);
            
            // Prendiamo TUTTI i messaggi per questo utente
            let messages_to_mark: Vec<_> = messages.iter().collect();
            
            for message in messages_to_mark {
                let client = Arc::clone(&client);
                let base_url = base_url.clone();
                let token = token.clone();
                let semaphore = Arc::clone(&semaphore);
                let message_id = message.id;
                
                mark_read_futures.push(async move {
                    let permit = semaphore.acquire().await.unwrap();
                    
                    let read_at = Utc::now();
                    let mark_read_payload = json!({
                        "text_message_id": message_id,
                        "read_at": read_at
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
                            Some((user_idx, message_id))
                        }
                        Ok(Ok(resp)) => {
                            eprintln!("❌ User {}: Mark read fallito per messaggio {} con status: {}", user_idx, message_id, resp.status());
                            None
                        }
                        Ok(Err(e)) => {
                            eprintln!("❌ User {}: Errore nella richiesta mark read per messaggio {}: {}", user_idx, message_id, e);
                            None
                        }
                        Err(_) => {
                            eprintln!("⏰ User {}: Timeout nella richiesta mark read per messaggio {}", user_idx, message_id);
                            None
                        }
                    }
                });
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

        cpu_usage_log_service.stop_monitoring().await.expect("Failed to stop monitoring");

        // Risultati
        let expected_mark_reads = NUM_USERS * NUM_MESSAGES; // Ogni utente dovrebbe marcare tutti i messaggi
        println!("📊 RISULTATI BENCHMARK LETTURA MESSAGGI:");
        println!("  Utenti lettori: {}", NUM_USERS);
        println!("  Messaggi totali nel gruppo: {}", NUM_MESSAGES);
        println!("  Tempo creazione messaggi: {:?}", message_creation_duration);
        println!("  Letture riuscite: {}/{}", successful_reads, NUM_USERS);
        println!("  Tempo lettura messaggi: {:?}", reading_duration);
        println!("  Tempo medio per lettura: {:?}", reading_duration / successful_reads);
        println!("  Letture/s: {:.2}", successful_reads as f64 / reading_duration.as_secs_f64());
        println!("  Messaggi totali letti: {}", total_messages_read);
        println!("  Media messaggi per utente: {:.2}", total_messages_read as f64 / successful_reads.max(1) as f64);
        println!("  Mark as read riuscite: {}/{}", successful_mark_reads, expected_mark_reads);
        println!("  Tempo mark as read: {:?}", mark_read_duration);
        println!("  Tempo medio per mark as read: {:?}", mark_read_duration / successful_mark_reads);
        println!("  Mark reads/s: {:.2}", successful_mark_reads as f64 / mark_read_duration.as_secs_f64());
        println!("  Success rate mark as read: {:.2}%", (successful_mark_reads as f64 / expected_mark_reads as f64) * 100.0);

        // Cleanup
        // Puliamo le info dei messaggi (read_at, sent_at)
        for message in &messages {
            cleanup_text_message_info_by_message_id(message.id).await;
        }
        
        // Puliamo i messaggi
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;

        // Puliamo gli utenti dal gruppo
        for user in &reader_users {
            cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        }
        cleanup_test_user_from_a_group_chat(admin_user.id, group.id).await;

        // Puliamo gruppo e utenti
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(admin_user.email).await;
        
        for user in reader_users {
            cleanup_user_by_email(user.email).await;
        }

        let _ = shutdown.send(());

        let total_duration = start_time.elapsed();
        println!("✅ Test completato in {:?}", total_duration);

        // Verifiche finali
        let read_success_rate = successful_reads as f64 / NUM_USERS as f64;
        let mark_read_success_rate = successful_mark_reads as f64 / expected_mark_reads as f64;
        
        assert_eq!(read_success_rate, 1.0, "Success rate lettura troppo basso: {:.2}%", read_success_rate * 100.0);
        assert_eq!(mark_read_success_rate, 1.0, "Success rate mark-as-read troppo basso: {:.2}%", mark_read_success_rate * 100.0);
        
        // Verifica che ogni utente abbia letto il numero corretto di messaggi
        for (user_idx, messages_count) in read_results {
            assert_eq!(messages_count, NUM_MESSAGES, "User {} dovrebbe aver letto {} messaggi, ma ne ha letti {}", user_idx, NUM_MESSAGES, messages_count);
        }
    }
}
