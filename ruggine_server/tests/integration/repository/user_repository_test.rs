use std::sync::Arc;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_repository_integration_tests {

    use ruggine_server::entity::user::{UserStatus};

    use crate::get_database;

    use super::*;

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        // Act
        let result = repository.find(99999).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_insert_duplicate_email() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        
        let user1 = UserFactory::unique_fake_new_user("duplicate", UserStatus::Active);
        let mut user2 = UserFactory::unique_fake_new_user("duplicate", UserStatus::Active);

        user2.email = user1.email.clone(); // Same email

        // Act
        let first_insert = repository.insert(user1.clone()).await;
        let second_insert = repository.insert(user2.clone()).await;

        // Assert
        assert!(first_insert.is_ok(), "First insert should succeed");
        assert!(second_insert.is_err(), "Second insert should fail due to unique constraint");

        // Cleanup
        if let Err(e) = repository.delete_by_email(user1.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user1.email.clone(), e);
        }
        assert!(repository.find_by_email(user1.email.clone()).await.is_none(), "User should be deleted");
    }

    #[tokio::test]
    async fn test_repository_concurrent_access() {
        use tokio::sync::Mutex;

        let db = get_database().await;
        let repository = Arc::new(UserRepository::new(&db));
        let emails_to_cleanup = Arc::new(Mutex::new(vec![]));

        let mut handles = vec![];

        // Spawn concurrent insertions
        for i in 0..5 {
            let repo_clone = Arc::clone(&repository);
            let emails_clone = Arc::clone(&emails_to_cleanup);

            let handle = tokio::spawn(async move {
                let new_user = UserFactory::unique_fake_new_user(&format!("concurrent_{}", i), UserStatus::Active);
                {
                    let mut guard = emails_clone.lock().await;
                    guard.push(new_user.email.clone());
                }
                repo_clone.insert(new_user).await
            });
            handles.push(handle);
        }

        // Wait for all inserts
        let results = futures::future::join_all(handles).await;

        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {i} panicked");
            assert!(result.as_ref().unwrap().is_ok(), "Insert {i} failed");
        }

        // ✅ CLEANUP PHASE: inline .await and make sure runtime is active
        let emails = emails_to_cleanup.lock().await.clone(); // Clone early to avoid holding lock during awaits

        let h = tokio::task::spawn(async move {
            for email in emails {
                if let Err(e) = repository.delete_by_email(email.clone()).await {
                    eprintln!("Cleanup failed for {}: {:?}", email, e);
                }
                match repository.find_by_email(email.clone()).await {
                    Some(_) => panic!("User should be deleted but still exists: {}", email),
                    None => {}
                }
            }
        });

        // Wait for cleanup to finish. This piece of code seems not necessary but due to some
        // problem of VS Code 'tokio' is closed before the cleanup task finishes.
        if let Err(e) = h.await {
            eprintln!("Cleanup task panicked: {:?}", e);
        }

    }
}
