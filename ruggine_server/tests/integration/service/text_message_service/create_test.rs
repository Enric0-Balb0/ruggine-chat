use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_create_message_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_msg_sender").await;
    
    // Create a test invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_msg_success_group",
        sender_user.id,
    ).await;

    let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

    // Act
    let result = service.create(payload.clone(), sender_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to create text message: {:?}", result.err());
    let created_message = result.unwrap();
    
    assert_eq!(created_message.content, payload.content);
    assert_eq!(created_message.sender_id, sender_user.id);
    assert_eq!(created_message.group_chat_id, group_chat.id);
    assert!(created_message.id > 0);

    // Cleanup
    common::cleanup_text_message(created_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_message_nonexistent_group() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_msg_no_group").await;
    let nonexistent_group_id = 99999;
    
    let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(nonexistent_group_id);

    // Act
    let result = service.create(payload, sender_user.id).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

    // Cleanup
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_message_user_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_msg_not_member").await;
    let (group_owner, _) = common::create_test_user("create_msg_group_owner").await;
    let group_chat = common::create_test_group_chat("create_msg_private_group", group_owner.id).await;
    
    let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

    // Act - sender_user is not a member of the group
    let result = service.create(payload, sender_user.id).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_message_user_not_active_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();
    let group_membership_service = service_init.group_membership_service();

    let (sender_user, _) = common::create_test_user("create_msg_long_content").await;

    // Create a test invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_msg_user_not_active_member_group",
        sender_user.id,
    ).await;

    let membership_id =group_membership_service
        .find_by_user_id_and_group_id(sender_user.id, group_chat.id)
        .await
        .expect("Should find the membership")
        .id;

    group_membership_service
        .leave_group(LeaveGroupMembershipDto {id: membership_id}, sender_user.id)
        .await
        .expect("Should be able to leave group");

    let mut payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);
    payload.content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".to_string();

    // Act
    let result = service.create(payload.clone(), sender_user.id).await;

    // Assert
    assert!(result.is_err(), "It should not allow to create a text message for inactive user: {:?}", result.err());

    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}


#[tokio_shared_rt::test(shared)]
async fn test_create_message_with_long_content() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_msg_long_content").await;

    // Create a test invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_msg_with_long_content_group",
        sender_user.id,
    ).await;

    let mut payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);
    payload.content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(50); // Long content

    // Act
    let result = service.create(payload.clone(), sender_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to create text message with long content: {:?}", result.err());
    let created_message = result.unwrap();
    
    assert_eq!(created_message.content, payload.content);
    assert_eq!(created_message.sender_id, sender_user.id);
    assert_eq!(created_message.group_chat_id, group_chat.id);

    // Cleanup
    common::cleanup_text_message(created_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_message_multiple_users_same_group() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (owner_user, _) = common::create_test_user("create_msg_owner").await;
    let (member_user, _) = common::create_test_user("create_msg_member").await;

    // Create a test invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_msg_with_long_content_group",
        owner_user.id,
    ).await;
    
    let member_membership = common::add_test_user_to_a_group(member_user.id, &group_chat).await;

    let owner_payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);
    let member_payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

    // Act - Both users create messages
    let owner_result = service.create(owner_payload.clone(), owner_user.id).await;
    let member_result = service.create(member_payload.clone(), member_user.id).await;

    // Assert
    assert!(owner_result.is_ok(), "Owner failed to create message: {:?}", owner_result.err());
    assert!(member_result.is_ok(), "Member failed to create message: {:?}", member_result.err());
    
    let owner_message = owner_result.unwrap();
    let member_message = member_result.unwrap();
    
    assert_eq!(owner_message.sender_id, owner_user.id);
    assert_eq!(member_message.sender_id, member_user.id);
    assert_eq!(owner_message.group_chat_id, group_chat.id);
    assert_eq!(member_message.group_chat_id, group_chat.id);
    assert_ne!(owner_message.id, member_message.id);

    // Cleanup
    common::cleanup_text_message(owner_message.id).await;
    common::cleanup_text_message(member_message.id).await;
    common::cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(owner_user.id).await;
    common::cleanup_user(member_user.id).await;
}
