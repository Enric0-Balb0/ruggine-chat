use crate::common;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_checked_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_checked_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_checked_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for find info checked".to_string())
    ).await;

    // Create text message info using the service
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Act
    let result = service.find_info_by_id_checked(created_info.id, sender_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message info: {:?}", result.err());
    let found_info = result.unwrap();
    
    assert_eq!(found_info.id, created_info.id);
    assert_eq!(found_info.user_id, sender_user.id);
    assert_eq!(found_info.text_message_id, text_message.id);

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_checked_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("find_info_checked_user").await;
    
    let non_existing_id = 999999;

    // Act
    let result = service.find_info_by_id_checked(non_existing_id, user.id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message info");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::MessageInfoNotFound) => {
            // Expected error type
        }
        _ => panic!("Expected MessageInfoNotFound error"),
    }

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_checked_access_denied_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_checked_sender").await;
    let (unauthorized_user, _) = common::create_test_user("find_info_checked_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_checked_access_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for access denied".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Act - Try to access with unauthorized user
    let result = service.find_info_by_id_checked(created_info.id, unauthorized_user.id).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(unauthorized_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_checked_access_denied_left_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_checked_sender").await;
    let (left_user, _) = common::create_test_user("find_info_checked_left").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_checked_left_group",
        sender_user.id,
    ).await;

    // Add left_user to the group then remove them
    let _left_membership = common::add_test_user_to_a_group(left_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for left member".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info1 = service.create_info(create_payload, sender_user.id).await.unwrap();

    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        left_user.id,
        text_message.id
    );
    let created_info2 = service.create_info(create_payload, sender_user.id).await.unwrap();

    // User leaves the group
    common::test_user_leave_from_a_group(left_user.id, group_chat.id).await;

    // Act - Try to access with left user
    let result = service.find_info_by_id_checked(created_info2.id, left_user.id).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error for left member");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        e => panic!("Expected UserCannotAccessMessages error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info1.id).await;
    common::cleanup_text_message_info(created_info2.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(left_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(left_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_checked_member_access_allowed() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_checked_sender").await;
    let (member_user, _) = common::create_test_user("find_info_checked_member").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_checked_member_group",
        sender_user.id,
    ).await;

    // Add member_user to the group
    let _member_membership = common::add_test_user_to_a_group(member_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for member access".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Act - Access with group member
    let result = service.find_info_by_id_checked(created_info.id, member_user.id).await;

    // Assert
    assert!(result.is_ok(), "Group member should be able to access message info: {:?}", result.err());
    let found_info = result.unwrap();
    
    assert_eq!(found_info.id, created_info.id);
    assert_eq!(found_info.text_message_id, text_message.id);

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(member_user.id).await;
}
