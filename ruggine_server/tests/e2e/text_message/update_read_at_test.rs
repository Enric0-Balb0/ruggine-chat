// e2e tests for /api/text_message/update_read_at endpoint
// Covers success, not found, unauthorized, already set, preserves sent_at, multiple users, cannot update again, cannot set before sent_at, must be >= sent_at, exactly equal, must be <= now
// Uses helpers from tests/common and factories

use chrono::{Utc, Duration};
use reqwest::StatusCode;
use serde_json::json;
use ruggine_server::dto::text_message_dto::{TextMessageInfoReadAtDtoUpdate, TextMessageInfoSentAtDtoUpdate};
use crate::common::*;
use crate::start_test_server;

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_success() {
    // Start server
    let (addr, shutdown) = start_test_server().await;

    // Create sender and recipient, login
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_recipient").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let sender_token = login_and_get_token_for_user(&sender, &sender_pw).await;

    // Create group, add both users
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;

    // Create message
    let message = create_test_text_message(sender.id, group.id, Some("e2e update_read_at message".to_string())).await;

    // Prepare sent_at and read_at
    let sent_at = Utc::now();
    let read_at = sent_at + Duration::milliseconds(10);
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Call API to update read_at
    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": read_at});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["data"]["text_message_id"], message.id);
    assert!(body["data"]["read_at"].as_str().is_some());

    // Cleanup
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}

// More tests to be added for not found, unauthorized, already set, etc.

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_message_not_found() {
    let (addr, shutdown) = start_test_server().await;
    let (user, pw) = create_test_user("e2e_update_read_at_not_found").await;
    let token = login_and_get_token_for_user(&user, &pw).await;
    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": 99999999, "read_at": Utc::now()});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    cleanup_user(user.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_unauthorized_user() {
    let (addr, shutdown) = start_test_server().await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_unauth").await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_recipient_unauth").await;
    let (unauth, unauth_pw) = create_test_user("e2e_update_read_at_unauthorized").await;
    let sender_token = login_and_get_token_for_user(&sender, &sender_pw).await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let unauth_token = login_and_get_token_for_user(&unauth, &unauth_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_unauth", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e unauthorized update_read_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;
    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": sent_at + Duration::milliseconds(10)});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&unauth_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    cleanup_user(unauth.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_already_set() {
    let (addr, shutdown) = start_test_server().await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_already_set").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_already_set").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_already_set", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e already set read_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": sent_at + Duration::milliseconds(10)});
    // First update
    let res1 = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    // Second update
    let res2 = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res2.status(), StatusCode::BAD_REQUEST);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_preserves_sent_at() {
    let (addr, shutdown) = start_test_server().await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_preserve").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_preserve").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_preserve", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e preserve sent_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": sent_at + Duration::milliseconds(10)});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["data"]["sent_at"].as_str().is_some());
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_multiple_users_same_message() {
    let (addr, shutdown) = start_test_server().await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_multi_sender").await;
    let (recipient1, recipient1_pw) = create_test_user("e2e_update_read_at_multi_recipient1").await;
    let (recipient2, recipient2_pw) = create_test_user("e2e_update_read_at_multi_recipient2").await;
    let recipient1_token = login_and_get_token_for_user(&recipient1, &recipient1_pw).await;
    let recipient2_token = login_and_get_token_for_user(&recipient2, &recipient2_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_multi", sender.id).await;
    let _ = add_test_user_to_a_group(recipient1.id, &group).await;
    let _ = add_test_user_to_a_group(recipient2.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e multi user read_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient1.id, message.id, sent_at).await;
    mark_message_as_sent(recipient2.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let payload1 = json!({"text_message_id": message.id, "read_at": sent_at + Duration::milliseconds(10)});
    let payload2 = json!({"text_message_id": message.id, "read_at": sent_at + Duration::milliseconds(20)});
    let res1 = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient1_token)
        .json(&payload1)
        .send().await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let res2 = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient2_token)
        .json(&payload2)
        .send().await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient1.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient2.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient1.id).await;
    cleanup_user(recipient2.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_cannot_set_before_sent_at() {
    let (addr, shutdown) = start_test_server().await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_before_sent").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_before_sent").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_before_sent", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e before sent_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": sent_at - Duration::milliseconds(10)});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_must_be_greater_or_equal_to_sent_at() {
    let (addr, shutdown) = start_test_server().await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_gte_sent").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_gte_sent").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_gte_sent", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e gte sent_at".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let payload = json!({"text_message_id": message.id, "read_at": sent_at});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_must_be_less_or_equal_to_now() {
    let (addr, shutdown) = start_test_server().await;
    let (recipient, recipient_pw) = create_test_user("e2e_update_read_at_lte_now").await;
    let (sender, sender_pw) = create_test_user("e2e_update_read_at_sender_lte_now").await;
    let recipient_token = login_and_get_token_for_user(&recipient, &recipient_pw).await;
    let group = create_test_group_chat_with_invitation_and_membership("e2e_update_read_at_lte_now", sender.id).await;
    let _ = add_test_user_to_a_group(recipient.id, &group).await;
    let message = create_test_text_message(sender.id, group.id, Some("e2e lte now".to_string())).await;
    let sent_at = Utc::now();
    mark_message_as_sent(recipient.id, message.id, sent_at).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let future_time = Utc::now() + Duration::seconds(10);
    let payload = json!({"text_message_id": message.id, "read_at": future_time});
    let res = client.patch(&format!("http://{}/api/text_message/update_read_at", addr))
        .bearer_auth(&recipient_token)
        .json(&payload)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    cleanup_text_message_info_by_message_id(message.id).await;
    cleanup_text_message(message.id).await;
    cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
    cleanup_test_user_from_a_group_chat(recipient.id, group.id).await;
    cleanup_group_chat(group.id).await;
    cleanup_user(sender.id).await;
    cleanup_user(recipient.id).await;
    shutdown.send(()).unwrap();
}
