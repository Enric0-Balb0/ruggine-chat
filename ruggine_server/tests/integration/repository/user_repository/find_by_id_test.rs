use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_repository_integration_tests {

    use ruggine_server::entity::user::{UserStatus};

    use crate::get_database;

    use super::*;


    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let new_user = UserFactory::unique_fake_new_user("findbyid", UserStatus::Active);

        // Act - Insert user first
        let user_id = repository.insert(new_user.clone()).await.unwrap();

        // Test find by id
        let found_user = repository.find(user_id).await;

        // Assert
        assert!(found_user.is_ok(), "Failed to find user by id");
        let user = found_user.unwrap();
        assert_eq!(user.id, user_id as i32); // Convert u64 to i32 for comparison
        assert_eq!(user.email, new_user.email.clone());

        // Cleanup
        if let Err(e) = repository.delete_by_email(new_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", new_user.email.clone(), e);
        }
        assert!(repository.find_by_email(new_user.email.clone()).await.is_none(), "User should be deleted");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        // Act
        let result = repository.find(99999).await;

        // Assert
        assert!(result.is_err());
    }
}
