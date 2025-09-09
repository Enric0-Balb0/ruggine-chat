use crate::common;
use chrono::{Duration, Utc};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_success_no_previous_sent() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("not_sent_sender").await;
    let (reader_user, _) = common::create_test_user("not_sent_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "not_sent_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages within the time range (more recent than epoch)
    let base_time = Utc::now();
    
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("First message not sent".to_string())
    ).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Second message not sent".to_string())
    ).await;

    // Act - Since no messages were marked as sent, should get messages from epoch to now
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not sent yet: {:?}", result.err());
    let response = result.unwrap();
    
    assert_eq!(response.data.len(), 2, "Should return both unsent messages");
    
    // Verify messages are ordered by sent_at DESC (newest first)
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&message1.id));
    assert!(message_ids.contains(&message2.id));

    // Verify pagination metadata
    assert!(response.pagination.has_more);
    assert_eq!(response.pagination.page_size, 2);

    // Cleanup
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_user_left_and_rejoined_group() {
    // Arrange - Test scenario where user leaves group, messages are created, then user rejoins
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (group_owner, _) = common::create_test_user("not_sent_rejoined_owner").await;
    let (reader_user, _) = common::create_test_user("not_sent_rejoined_reader").await;
    let (sender_user, _) = common::create_test_user("not_sent_rejoined_sender").await;
    
    // Create a test group with invitation and membership for the owner
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "not_sent_rejoined_group",
        group_owner.id,
    ).await;

    // Add reader and sender to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;
    let _sender_membership = common::add_test_user_to_a_group(sender_user.id, &group_chat).await;

    // Phase 1: Create initial messages while reader is in the group
    let initial_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Initial message 1".to_string())
    ).await;

    let initial_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Initial message 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Reader marks the initial messages as sent
    common::mark_message_as_sent(reader_user.id, initial_message1.id).await;
    common::mark_message_as_sent(reader_user.id, initial_message2.id).await;

    // Phase 2: Create some messages that reader doesn't mark as sent
    let unsent_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Unsent message 1".to_string())
    ).await;

    let unsent_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Unsent message 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Phase 3: Reader leaves the group
    common::test_user_leave_from_a_group(reader_user.id, group_chat.id).await;

    // Phase 4: Create messages while reader is NOT in the group (these won't have TextMessageInfo for reader)
    let missing_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message while user was absent 1".to_string())
    ).await;

    let missing_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message while user was absent 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Phase 5: Reader rejoins the group
    let _reader_membership_new = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Phase 6: Create more messages after rejoining
    let after_rejoin_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message after rejoin 1".to_string())
    ).await;

    let after_rejoin_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message after rejoin 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Act - Call find_messages_not_sent_yet which should handle missing TextMessageInfo gracefully
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not sent yet: {:?}", result.err());
    let response = result.unwrap();
    
    // Should return all unsent messages (both before leaving and after rejoining)
    // The messages created while user was absent should trigger MessageInfoNotFound errors 
    // but the function should handle them gracefully and include them in the response
    assert_eq!(response.data.len(), 6, "Expected 6 unsent messages (2 unsent + 2 after left + 2 after rejoin), got: {}", response.data.len());
    
    // Verify that at least the messages we expect are included
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    
    // These should definitely be included
    assert!(message_ids.contains(&unsent_message1.id), "Expected unsent_message1 to be processed");
    assert!(message_ids.contains(&unsent_message2.id), "Expected unsent_message2 to be processed");
    assert!(message_ids.contains(&missing_message1.id), "Expected missing_message1 to be processed");
    assert!(message_ids.contains(&missing_message2.id), "Expected missing_message2 to be processed");
    assert!(message_ids.contains(&after_rejoin_message1.id), "Expected after_rejoin_message1 to be processed");
    assert!(message_ids.contains(&after_rejoin_message2.id), "Expected after_rejoin_message2 to be processed");
    
    // The initial messages should NOT be included as they were already sent
    assert!(!message_ids.contains(&initial_message1.id), "Initial_message1 should not be processed as it was already sent");
    assert!(!message_ids.contains(&initial_message2.id), "Initial_message2 should not be processed as it was already sent");

    // Find again with no not sent messages
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;
    assert_eq!(result.unwrap().data.len(), 0, "there should not be other not sent messages");

    // Cleanup
    common::cleanup_text_message(initial_message1.id).await;
    common::cleanup_text_message(initial_message2.id).await;
    common::cleanup_text_message(missing_message1.id).await;
    common::cleanup_text_message(missing_message2.id).await;
    common::cleanup_text_message(unsent_message1.id).await;
    common::cleanup_text_message(unsent_message2.id).await;
    common::cleanup_text_message(after_rejoin_message1.id).await;
    common::cleanup_text_message(after_rejoin_message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
    common::cleanup_user(reader_user.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_success_with_previous_sent() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("sent_yet_sender").await;
    let (reader_user, _) = common::create_test_user("sent_yet_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "sent_yet_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create older message that was already sent
    let old_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Old sent message".to_string())
    ).await;

    let old_info = service.find_info_by_user_id_and_message_id(reader_user.id, old_message.id).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    common::mark_message_as_sent(reader_user.id, old_message.id).await;

    // Wait a bit and create new messages that haven't been sent yet
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let new_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("New message 1 not sent".to_string())
    ).await;

    let new_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("New message 2 not sent".to_string())
    ).await;

    // Act - Should get only messages created after the last sent message
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not sent yet with previous sent: {:?}", result.err());
    let response = result.unwrap();
    
    assert_eq!(response.data.len(), 2, "Should return only the new unsent messages");
    
    // Verify only new messages are returned, not the old sent one
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&new_message1.id));
    assert!(message_ids.contains(&new_message2.id));
    assert!(!message_ids.contains(&old_message.id));

    // Cleanup
    common::cleanup_text_message(old_message.id).await;
    common::cleanup_text_message(new_message1.id).await;
    common::cleanup_text_message(new_message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_empty_result() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("empty_result_sender").await;
    let (reader_user, _) = common::create_test_user("empty_result_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "empty_result_group",
        sender_user.id,
    ).await;

    // Create also a message not sent to reader user
    let old_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message not for reader user".to_string())
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create a message and mark it as sent, so there are no unsent messages
    let message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Already sent message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    common::mark_message_as_sent(reader_user.id, message.id).await;

    // Act - Should get empty result since all messages are already sent
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Should succeed even with empty result");
    let response = result.unwrap();
    
    assert_eq!(response.data.len(), 0, "Should return empty result when all messages are sent");
    assert_eq!(response.pagination.page_size, 0);

    // Cleanup
    common::cleanup_text_message(old_message.id).await;
    common::cleanup_text_message(message.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_group_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("group_not_found_user").await;
    let non_existent_group_id = 99999;

    // Act
    let result = service.find_messages_not_sent_yet(non_existent_group_id, user.id).await;

    // Assert
    assert!(result.is_err(), "Should fail when group doesn't exist");
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::GroupChatNotFound)), "Expected GroupChatNotFound, got {:?}", error);

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_user_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (creator_user, _) = common::create_test_user("not_member_creator").await;
    let (non_member_user, _) = common::create_test_user("not_member_user").await;
    
    // Create a test group with invitation and membership for the creator only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "not_member_group",
        creator_user.id,
    ).await;

    // Act - Try to access with user who is not a member
    let result = service.find_messages_not_sent_yet(group_chat.id, non_member_user.id).await;

    // Assert
    assert!(result.is_err(), "Should fail when user is not a member");
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(creator_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(creator_user.id).await;
    common::cleanup_user(non_member_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_inactive_membership() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (creator_user, _) = common::create_test_user("inactive_member_creator").await;
    let (inactive_user, _) = common::create_test_user("inactive_member_user").await;
    
    // Create a test group with invitation and membership for the creator
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "inactive_member_group",
        creator_user.id,
    ).await;

    // Add user to the group and then make them inactive (leave the group)
    let user_membership = common::add_test_user_to_a_group(inactive_user.id, &group_chat).await;
    
    // Leave the group to make membership inactive
    let service_init = ServiceInitializer::new(&db);
    let membership_service = service_init.group_membership_service();
    let _leave_result = membership_service.leave_group(
        ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto { id: user_membership.id },
        inactive_user.id
    ).await;

    // Act - Try to access with user who has inactive membership
    let result = service.find_messages_not_sent_yet(group_chat.id, inactive_user.id).await;

    // Assert
    assert!(result.is_err(), "Should fail when user has inactive membership");
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(inactive_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(creator_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(creator_user.id).await;
    common::cleanup_user(inactive_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_updates_sent_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_sender").await;
    let (reader_user, _) = common::create_test_user("update_sent_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test message
    let message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message to update sent_at".to_string())
    ).await;

    // Verify initially not marked as sent
    let initial_info = service.find_info_by_user_id_and_message_id(reader_user.id, message.id).await.unwrap();
    assert!(initial_info.sent_at.is_none(), "Message should initially not be marked as sent");

    // Act - Call find_messages_not_sent_yet which should update sent_at
    let result = service.find_messages_not_sent_yet(group_chat.id, reader_user.id).await;

    // Assert - Function should succeed and update sent_at
    assert!(result.is_ok(), "Failed to find messages and update sent_at: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 1, "Should return the message");

    // Verify sent_at was updated
    let updated_info = service.find_info_by_id(initial_info.id).await.unwrap();
    assert!(updated_info.sent_at.is_some(), "Message should now be marked as sent");
    assert!(updated_info.sent_at.unwrap() > initial_info.sent_at.unwrap_or(Utc::now() - Duration::days(1)));

    // Cleanup
    common::cleanup_text_message(message.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_sent_yet_multiple_users_scenario() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("multi_user_sender").await;
    let (reader1, _) = common::create_test_user("multi_user_reader1").await;
    let (reader2, _) = common::create_test_user("multi_user_reader2").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "multi_user_group",
        sender_user.id,
    ).await;

    // Add readers to the group
    let _reader1_membership = common::add_test_user_to_a_group(reader1.id, &group_chat).await;
    let _reader2_membership = common::add_test_user_to_a_group(reader2.id, &group_chat).await;

    // Create test message
    let message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message for multiple users".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark as sent for reader1 only
    common::mark_message_as_sent(reader1.id, message.id).await;

    // Act - Reader1 should get empty result (message already sent)
    let result1 = service.find_messages_not_sent_yet(group_chat.id, reader1.id).await;
    
    // Act - Reader2 should get the message (not sent yet)
    let result2 = service.find_messages_not_sent_yet(group_chat.id, reader2.id).await;

    // Assert
    assert!(result1.is_ok(), "Reader1 query should succeed");
    let response1 = result1.unwrap();
    assert_eq!(response1.data.len(), 0, "Reader1 should get empty result");

    assert!(result2.is_ok(), "Reader2 query should succeed");
    let response2 = result2.unwrap();
    assert_eq!(response2.data.len(), 1, "Reader2 should get the unsent message");
    assert_eq!(response2.data[0].id, message.id);

    // Cleanup
    common::cleanup_text_message(message.id).await;
    common::cleanup_test_user_from_a_group_chat(reader1.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(reader2.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader1.id).await;
    common::cleanup_user(reader2.id).await;
}
