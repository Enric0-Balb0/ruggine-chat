use sqlx::Error;

// Helper struct to mock database errors
#[derive(Debug)]
pub struct MockDatabaseError {
    code: String,
    message: String,
}

impl MockDatabaseError {
    pub fn new(code: String) -> Self {
        Self {
            code: code.clone(),
            message: format!("Mock database error with code: {}", code),
        }
    }

    pub fn with_message(code: String, message: String) -> Self {
        Self {
            code,
            message,
        }
    }

    // PostgreSQL foreign key violation code
    pub fn foreign_key_violation() -> Error {
        Error::Database(Box::new(Self {
            code: "23503".to_string(),
            message: "Foreign key violation 23503".to_string(),
        }))
    }

    // PostgreSQL constraint violation code
    pub fn constraint_violation() -> Error {
        Error::Database(Box::new(Self {
            code: "23505".to_string(),
            message: "Constraint violation 23505".to_string(),
        }))
    }
}

impl std::fmt::Display for MockDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MockDatabaseError {}

impl sqlx::error::DatabaseError for MockDatabaseError {
    fn message(&self) -> &str {
        &self.message
    }

    fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
        Some(std::borrow::Cow::Borrowed(&self.code))
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}