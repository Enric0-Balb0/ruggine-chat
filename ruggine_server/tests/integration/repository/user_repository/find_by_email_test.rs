use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_repository_integration_tests {
    use ruggine_server::entity::user::{UserStatus};

    use crate::get_database;

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_find_and_delete_by_email() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let new_user = UserFactory::unique_fake_new_user("insertfind", UserStatus::Active);

        // Act
        let insert_result = repository.insert(new_user.clone()).await;

        // Assert
        assert!(insert_result.is_ok(), "Failed to insert user: {:?}", insert_result);
        let user_id = insert_result.unwrap();
        assert!(user_id > 0);

        // Test find_by_email
        let found_user = repository.find_by_email(new_user.email.clone()).await;
        assert!(found_user.is_some(), "User not found by email");

        let user = found_user.unwrap();
        assert_eq!(user.email, new_user.email);
        assert_eq!(user.first_name, new_user.first_name);
        assert_eq!(user.last_name, new_user.last_name);

        // Cleanup
        if let Err(e) = repository.delete_by_email(new_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", new_user.email, e);
        }
        assert!(repository.find_by_email(new_user.email.clone()).await.is_none(), "User should be deleted");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_email_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let (nonexistent_email, _, _) = UserFactory::get_unique_user_information("nonexistent");

        // Act
        let result = repository.find_by_email(nonexistent_email).await;

        // Assert
        assert!(result.is_none(), "Expected no user to be found");
    }
}