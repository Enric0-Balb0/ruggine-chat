use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_insert_text_message_info_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group first, then a text message
    let (sender_user, _) = common::create_test_user("text_msg_info_sender").await;
    let (recipient_user, _) = common::create_test_user("text_msg_info_recipient").await;
    let group_chat = common::create_test_group_chat("text_msg_info_group", sender_user.id).await;
    
    // Create a text message first
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for info".to_string())
    ).await;
    
    let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );

    // Act
    let result = repository.insert_text_message_info(text_message_info.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message info: {:?}", result.err());
    let info_id = result.unwrap();
    assert!(info_id > 0, "Message info ID should be positive");

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_text_message_info_foreign_key_violation_user() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test user and group, then a text message
    let (sender_user, _) = common::create_test_user("text_msg_fk_sender").await;
    let group_chat = common::create_test_group_chat("text_msg_fk_group", sender_user.id).await;
    
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for FK violation".to_string())
    ).await;
    
    // Use non-existent user_id but valid text_message_id
    let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        999999, // Non-existent user
        text_message.id
    );

    // Act
    let result = repository.insert_text_message_info(text_message_info).await;

    // Assert
    assert!(result.is_err(), "Expected foreign key constraint violation");
    let error = result.unwrap_err();
    assert!(
        matches!(error, sqlx::Error::Database(_)),
        "Expected database error for foreign key violation"
    );

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_text_message_info_foreign_key_violation_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test user only
    let (user, _) = common::create_test_user("text_msg_info_fk_user").await;
    
    // Use valid user_id but non-existent text_message_id
    let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        user.id,
        999999 // Non-existent text message
    );

    // Act
    let result = repository.insert_text_message_info(text_message_info).await;

    // Assert
    assert!(result.is_err(), "Expected foreign key constraint violation");
    let error = result.unwrap_err();
    assert!(
        matches!(error, sqlx::Error::Database(_)),
        "Expected database error for foreign key violation"
    );

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_text_message_info_multiple_users_same_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group, then a text message
    let (sender_user, _) = common::create_test_user("multi_info_sender").await;
    let (recipient1, _) = common::create_test_user("multi_info_recipient1").await;
    let (recipient2, _) = common::create_test_user("multi_info_recipient2").await;
    let group_chat = common::create_test_group_chat("multi_info_group", sender_user.id).await;
    
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Message with multiple info records".to_string())
    ).await;
    
    // Create text message info for multiple users
    let info1 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient1.id,
        text_message.id
    );
    let info2 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient2.id,
        text_message.id
    );

    // Act
    let result1 = repository.insert_text_message_info(info1).await;
    let result2 = repository.insert_text_message_info(info2).await;

    // Assert
    assert!(result1.is_ok(), "Failed to insert first text message info");
    assert!(result2.is_ok(), "Failed to insert second text message info");
    
    let info_id1 = result1.unwrap();
    let info_id2 = result2.unwrap();
    
    assert!(info_id1 > 0, "First message info ID should be positive");
    assert!(info_id2 > 0, "Second message info ID should be positive");
    assert_ne!(info_id1, info_id2, "Message info IDs should be different");

    // Cleanup
    common::cleanup_text_message_info(info_id1).await;
    common::cleanup_text_message_info(info_id2).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1.id).await;
    common::cleanup_user(recipient2.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_text_message_info_with_factory_utility_methods() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group, then a text message
    let (sender_user, _) = common::create_test_user("factory_util_sender").await;
    let (recipient_user, _) = common::create_test_user("factory_util_recipient").await;
    let group_chat = common::create_test_group_chat("factory_util_group", sender_user.id).await;
    
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for factory utilities".to_string())
    ).await;
    
    // Use factory utility methods to create text message info
    let base_info = TextMessageFactory::fake_new_text_message_info();
    let text_message_info = TextMessageFactory::with_user_id_info(
        TextMessageFactory::with_text_message_id_info(base_info, text_message.id),
        recipient_user.id
    );

    // Act
    let result = repository.insert_text_message_info(text_message_info).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message info using factory utilities");
    let info_id = result.unwrap();
    assert!(info_id > 0, "Message info ID should be positive");

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}
