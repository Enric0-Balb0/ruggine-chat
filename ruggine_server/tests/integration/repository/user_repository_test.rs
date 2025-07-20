use std::sync::Arc;
use ruggine_server::config::database::{Database, DatabaseTrait};
use ruggine_server::entity::user::NewUser;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use std::sync::atomic::{AtomicU32, Ordering};

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

// Helper function to generate unique test data
fn get_unique_test_data(prefix: &str) -> (String, String, String) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let unique_suffix = format!("{}_{}", counter, timestamp);
    let email = format!("{}{}@test.com", prefix, unique_suffix);
    let username = format!("{}_{}", prefix, unique_suffix);
    let full_name = format!("{}{}", prefix, unique_suffix);
    
    (email, username, full_name)
}

// Helper function for test database setup
async fn setup_test_db() -> Arc<Database> {
    // In a real environment, you'd use a separate test database
    // For now, we use the standard database initialization
    // You can set TEST_DATABASE_URL environment variable for a test-specific database
    dotenv::dotenv().ok();
    let _database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/ruggine_test".to_string());
    println!("DB URL: {}", _database_url);
    
    // Note: Database::init() should be used instead of new_with_url
    // For integration tests, you'd need to set up the database properly
    let database = Database::init(_database_url).await
        .expect("Failed to connect to test database");
    
    Arc::new(database)
}

// Helper function to clean database after tests - now cleans only specific test data
async fn cleanup_test_db_specific(db: &Arc<Database>, email: &str) {
    if let Err(e) = sqlx::query("DELETE FROM user WHERE email = ?")
        .bind(email)
        .execute(db.get_pool())
        .await 
    {
        eprintln!("Failed to cleanup test data for {}: {}", email, e);
    }
}

#[cfg(test)]
mod user_repository_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_find_by_email() {
        // Arrange
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);
        
        let (email, username, full_name) = get_unique_test_data("integration");
        let new_user = NewUser {
            first_name: Some(full_name.clone()),
            last_name: Some("Test".to_string()),
            user_name: username.clone(),
            email: email.clone(),
            password: "hashed_password".to_string(),
            is_active: 1, // i8 value (1 for active)
        };

        // Act
        let insert_result = repository.insert(new_user).await;
        
        // Assert
        assert!(insert_result.is_ok(), "Failed to insert user: {:?}", insert_result);
        let user_id = insert_result.unwrap();
        assert!(user_id > 0);

        // Test find_by_email
        let found_user = repository.find_by_email(email.clone()).await;
        assert!(found_user.is_some(), "User not found by email");
        
        let user = found_user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.first_name, Some(full_name));
        assert_eq!(user.last_name, Some("Test".to_string()));

        // Cleanup
        cleanup_test_db_specific(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        // Arrange
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);
        
        let (email, username, full_name) = get_unique_test_data("findbyid");
        let new_user = NewUser {
            first_name: Some(full_name),
            last_name: Some("ById".to_string()),
            user_name: username,
            email: email.clone(),
            password: "hashed_password".to_string(),
            is_active: 1, // i8 value (1 for active)
        };

        // Act - Insert user first
        let user_id = repository.insert(new_user).await.unwrap();
        
        // Test find by id  
        let found_user = repository.find(user_id).await;
        
        // Assert
        assert!(found_user.is_ok(), "Failed to find user by id");
        let user = found_user.unwrap();
        assert_eq!(user.id, user_id as i32); // Convert u64 to i32 for comparison
        assert_eq!(user.email, email);

        // Cleanup
        cleanup_test_db_specific(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_email_not_found() {
        // Arrange
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);
        
        let (nonexistent_email, _, _) = get_unique_test_data("nonexistent");

        // Act
        let result = repository.find_by_email(nonexistent_email).await;

        // Assert
        assert!(result.is_none(), "Expected no user to be found");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);

        // Act
        let result = repository.find(99999).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_insert_duplicate_email() {
        // Arrange
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);
        
        let (shared_email, username1, full_name1) = get_unique_test_data("duplicate");
        let (_, username2, full_name2) = get_unique_test_data("duplicate2");
        
        let user1 = NewUser {
            first_name: Some(full_name1),
            last_name: Some("User".to_string()),
            user_name: username1,
            email: shared_email.clone(),
            password: "password1".to_string(),
            is_active: 1, // i8 value (1 for active)
        };

        let user2 = NewUser {
            first_name: Some(full_name2),
            last_name: Some("User".to_string()),
            user_name: username2,
            email: shared_email.clone(), // Same email
            password: "password2".to_string(),
            is_active: 1, // i8 value (1 for active)
        };

        // Act
        let first_insert = repository.insert(user1).await;
        let second_insert = repository.insert(user2).await;

        // Assert
        assert!(first_insert.is_ok(), "First insert should succeed");
        assert!(second_insert.is_err(), "Second insert should fail due to unique constraint");

        // Cleanup
        cleanup_test_db_specific(&db, &shared_email).await;
    }

    #[tokio::test]
    async fn test_repository_concurrent_access() {
        // Test to verify concurrent access
        let db = setup_test_db().await;
        let repository = Arc::new(UserRepository::new(&db));

        let mut handles = vec![];
        let mut emails_to_cleanup = vec![];

        // Create 5 concurrent tasks
        for i in 0..5 {
            let repo_clone: Arc<UserRepository> = Arc::clone(&repository);
            let (email, username, full_name) = get_unique_test_data(&format!("concurrent{}", i));
            emails_to_cleanup.push(email.clone());
            
            let handle = tokio::spawn(async move {
                let new_user = NewUser {
                    first_name: Some(full_name),
                    last_name: Some("Test".to_string()),
                    user_name: username,
                    email,
                    password: "password".to_string(),
                    is_active: 1, // i8 value (1 for active)
                };

                repo_clone.insert(new_user).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to finish
        let results: Vec<_> = futures::future::join_all(handles).await;

        // Verify that all insertions succeeded
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} panicked", i);
            assert!(result.as_ref().unwrap().is_ok(), "Insert {} failed", i);
        }

        // Cleanup
        for email in emails_to_cleanup {
            cleanup_test_db_specific(&db, &email).await;
        }
    }
}

#[cfg(test)]
mod user_repository_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_performance() {
        let db = setup_test_db().await;
        let repository = UserRepository::new(&db);
        
        use std::time::Instant;
        
        let start = Instant::now();
        let mut emails_to_cleanup = vec![];
        
        // Insert 100 users
        for i in 0..100 {
            let (email, username, full_name) = get_unique_test_data(&format!("perf{}", i));
            emails_to_cleanup.push(email.clone());
            
            let new_user = NewUser {
                first_name: Some(full_name),
                last_name: Some("Test".to_string()),
                user_name: username,
                email,
                password: "password".to_string(),
                is_active: 1, // i8 value (1 for active)
            };

            repository.insert(new_user).await.unwrap();
        }

        let duration = start.elapsed();
        println!("100 insertions took: {:?}", duration);
        
        // Assert that it doesn't take more than 10 seconds
        assert!(duration.as_secs() < 10, "Performance test took too long: {:?}", duration);

        // Cleanup
        for email in emails_to_cleanup {
            cleanup_test_db_specific(&db, &email).await;
        }
    }
}
