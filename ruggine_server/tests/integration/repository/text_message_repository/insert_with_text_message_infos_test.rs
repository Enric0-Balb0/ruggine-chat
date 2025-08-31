use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_success_single_member() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group with membership
    let (sender_user, _) = common::create_test_user("msg_infos_sender").await;
    let (member_user, _) = common::create_test_user("msg_infos_member").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("msg_infos_group", sender_user.id).await;
    
    // Add member to the group
    let membership = common::add_test_user_to_a_group(member_user.id, &group_chat).await;
    
    let new_message = TextMessageFactory::unique_fake_new_text_message(
        "test_with_infos",
        sender_user.id,
        group_chat.id
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message with infos: {:?}", result.err());
    let message_id = result.unwrap();
    assert!(message_id > 0, "Message ID should be positive");

    // Verify the message was actually inserted
    let inserted_message = repository.find(message_id).await;
    assert!(inserted_message.is_ok(), "Failed to find inserted message");
    
    let message = inserted_message.unwrap();
    assert_eq!(message.content, new_message.content);
    assert_eq!(message.sender_id, new_message.sender_id);
    assert_eq!(message.group_chat_id, new_message.group_chat_id);

    // Verify text_message_info records were created for group members
    let message_infos = repository.find_info_by_message_id(message_id).await;
    assert!(message_infos.is_ok(), "Failed to find message infos");
    
    let infos = message_infos.unwrap();
    // Should have info records for all active group members (sender and member)
    assert_eq!(infos.len(), 2, "Should create info records for all active group members");
    
    // Verify both users have info records
    let user_ids: Vec<i32> = infos.iter().map(|info| info.user_id).collect();
    assert!(user_ids.contains(&sender_user.id), "Sender should have message info");
    assert!(user_ids.contains(&member_user.id), "Member should have message info");

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(member_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_success_multiple_members() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group
    let (sender_user, _) = common::create_test_user("multi_infos_sender").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("multi_infos_group", sender_user.id).await;
    
    // Add multiple members to the group
    let members = common::create_test_users_for_a_group("multi_infos_member", 3, &group_chat).await;
    
    let new_message = TextMessageFactory::unique_fake_new_text_message(
        "test_multi_infos",
        sender_user.id,
        group_chat.id
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message with multiple infos: {:?}", result.err());
    let message_id = result.unwrap();
    assert!(message_id > 0, "Message ID should be positive");

    // Verify the message was actually inserted
    let inserted_message = repository.find(message_id).await;
    assert!(inserted_message.is_ok(), "Failed to find inserted message");
    
    let message = inserted_message.unwrap();
    assert_eq!(message.content, new_message.content);
    assert_eq!(message.sender_id, new_message.sender_id);
    assert_eq!(message.group_chat_id, new_message.group_chat_id);

    // Verify text_message_info records were created for all group members
    let message_infos = repository.find_info_by_message_id(message_id).await;
    assert!(message_infos.is_ok(), "Failed to find message infos");
    
    let infos = message_infos.unwrap();
    // Should have info records for sender + 3 members = 4 total
    assert_eq!(infos.len(), 4, "Should create info records for all active group members");
    
    // Verify all users have info records
    let user_ids: Vec<i32> = infos.iter().map(|info| info.user_id).collect();
    assert!(user_ids.contains(&sender_user.id), "Sender should have message info");
    
    for (member, _, _) in &members {
        assert!(user_ids.contains(&member.id), "Member {} should have message info", member.id);
    }

    // Cleanup
    common::cleanup_text_message(message_id).await;
    let member_ids: Vec<i32> = members.iter().map(|(user, _, _)| user.id).collect();
    common::cleanup_test_users_from_a_group_chat(member_ids, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    let user_ids: Vec<i32> = members.iter().map(|(user, _, _)| user.id).collect();
    common::cleanup_test_users(user_ids).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_success_sender_only() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test user and group with no additional members
    let (sender_user, _) = common::create_test_user("sender_only_sender").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("sender_only_group", sender_user.id).await;
    
    let new_message = TextMessageFactory::unique_fake_new_text_message(
        "test_sender_only",
        sender_user.id,
        group_chat.id
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message with sender only: {:?}", result.err());
    let message_id = result.unwrap();
    assert!(message_id > 0, "Message ID should be positive");

    // Verify the message was actually inserted
    let inserted_message = repository.find(message_id).await;
    assert!(inserted_message.is_ok(), "Failed to find inserted message");
    
    let message = inserted_message.unwrap();
    assert_eq!(message.content, new_message.content);
    assert_eq!(message.sender_id, new_message.sender_id);
    assert_eq!(message.group_chat_id, new_message.group_chat_id);

    // Verify text_message_info record was created for sender
    let message_infos = repository.find_info_by_message_id(message_id).await;
    assert!(message_infos.is_ok(), "Failed to find message infos");
    
    let infos = message_infos.unwrap();
    // Should have info record for sender only
    assert_eq!(infos.len(), 1, "Should create info record for sender");
    
    // Verify sender has info record
    assert_eq!(infos[0].user_id, sender_user.id, "Sender should have message info");

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_foreign_key_violation_sender() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create group but use non-existent sender
    let (group_owner, _) = common::create_test_user("fk_violation_owner").await;
    let group_chat = common::create_test_group_chat("fk_violation_group", group_owner.id).await;
    
    let new_message = TextMessageFactory::fake_new_text_message_with_ids(
        999999, // Non-existent sender
        group_chat.id
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message).await;

    // Assert
    assert!(result.is_err(), "Expected foreign key constraint violation for non-existent sender");

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_foreign_key_violation_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create user but use non-existent group
    let (sender_user, _) = common::create_test_user("fk_group_sender").await;
    
    let new_message = TextMessageFactory::fake_new_text_message_with_ids(
        sender_user.id,
        999999 // Non-existent group
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message).await;

    // Assert
    assert!(result.is_err(), "Expected foreign key constraint violation for non-existent group");

    // Cleanup
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_with_long_content() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group
    let (sender_user, _) = common::create_test_user("long_content_sender").await;
    let (member_user, _) = common::create_test_user("long_content_member").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("long_content_group", sender_user.id).await;
    
    // Add member to the group
    let membership = common::add_test_user_to_a_group(member_user.id, &group_chat).await;
    
    let long_content = "a".repeat(5000); // Long but reasonable content
    let new_message = TextMessageFactory::fake_new_text_message_with_content(long_content.clone());
    let new_message = TextMessageFactory::with_sender_id(new_message, sender_user.id);
    let new_message = TextMessageFactory::with_group_chat_id(new_message, group_chat.id);

    // Act
    let result = repository.insert_with_text_message_infos(new_message).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert message with long content and infos");
    let message_id = result.unwrap();
    
    let inserted_message = repository.find(message_id).await.unwrap();
    assert_eq!(inserted_message.content, long_content);

    // Verify message infos were created
    let message_infos = repository.find_info_by_message_id(message_id).await.unwrap();
    assert_eq!(message_infos.len(), 2, "Should create info records for both users");

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(member_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_transaction_rollback_on_info_failure() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group
    let (sender_user, _) = common::create_test_user("rollback_sender").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("rollback_group", sender_user.id).await;
    
    let new_message = TextMessageFactory::unique_fake_new_text_message(
        "test_rollback",
        sender_user.id,
        group_chat.id
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message.clone()).await;

    // Assert success first
    assert!(result.is_ok(), "Message insertion should succeed");
    let message_id = result.unwrap();

    // Verify message exists
    let message_exists = repository.find(message_id).await;
    assert!(message_exists.is_ok(), "Message should exist in database");

    // Verify message infos exist
    let infos_exist = repository.find_info_by_message_id(message_id).await;
    assert!(infos_exist.is_ok(), "Message infos should exist");
    assert!(!infos_exist.unwrap().is_empty(), "Message infos should not be empty");

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_text_message_infos_with_factory_utility_methods() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group
    let (sender_user, _) = common::create_test_user("factory_util_sender").await;
    let (member_user, _) = common::create_test_user("factory_util_member").await;
    let group_chat = common::create_test_group_chat_with_invitation_and_membership("factory_util_group", sender_user.id).await;
    
    // Add member to the group
    let membership = common::add_test_user_to_a_group(member_user.id, &group_chat).await;
    
    // Use factory utility methods to create text message
    let base_message = TextMessageFactory::fake_new_text_message();
    let new_message = TextMessageFactory::with_content(
        TextMessageFactory::with_sender_id(
            TextMessageFactory::with_group_chat_id(base_message, group_chat.id),
            sender_user.id
        ),
        "Factory utility test message".to_string()
    );

    // Act
    let result = repository.insert_with_text_message_infos(new_message.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message using factory utilities");
    let message_id = result.unwrap();
    assert!(message_id > 0, "Message ID should be positive");

    // Verify message content
    let inserted_message = repository.find(message_id).await.unwrap();
    assert_eq!(inserted_message.content, "Factory utility test message");
    assert_eq!(inserted_message.sender_id, sender_user.id);
    assert_eq!(inserted_message.group_chat_id, group_chat.id);

    // Verify message infos
    let message_infos = repository.find_info_by_message_id(message_id).await.unwrap();
    assert_eq!(message_infos.len(), 2, "Should create info records for both users");

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(member_user.id).await;
}
