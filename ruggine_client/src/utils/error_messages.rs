use crate::error::AuthError;
use crate::api::http_error::HttpError;

/// Convert AuthError to user-friendly Italian messages for login context
pub fn auth_error_to_login_message(error: &AuthError) -> String {
    match error {
        AuthError::InvalidCredentials => "Email o password non validi".to_string(),
        AuthError::InvalidInput(msg) => format!("Dati non validi: {}", msg),
        AuthError::ValidationError(msg) => format!("Errore di validazione: {}", msg),
        AuthError::NetworkError(msg) => format!("Errore di connessione: {}", msg),
        AuthError::NotAuthenticated => "Sessione scaduta, effettua nuovamente il login".to_string(),
        AuthError::Http(http_error) => http_error_to_login_message(http_error),
        AuthError::Storage(_) => "Errore nel salvataggio dei dati. Riprova.".to_string(),
    }
}

/// Convert AuthError to user-friendly Italian messages for registration context
pub fn auth_error_to_register_message(error: &AuthError) -> String {
    match error {
        AuthError::InvalidInput(msg) => msg.clone(),
        AuthError::ValidationError(msg) => format!("Errore di validazione: {}", msg),
        AuthError::NetworkError(msg) => format!("Errore di connessione: {}", msg),
        AuthError::Http(http_error) => http_error_to_register_message(http_error),
        AuthError::Storage(_) => "Errore nel salvataggio dei dati. Riprova.".to_string(),
        _ => "Errore durante la registrazione".to_string(),
    }
}

/// Convert HttpError to user-friendly messages for login context
fn http_error_to_login_message(http_error: &HttpError) -> String {
    match http_error {
        HttpError::Timeout => "La richiesta ha impiegato troppo tempo. Verifica la connessione.".to_string(),
        HttpError::Network(msg) => format!("Errore di rete: {}", msg),
        HttpError::Server(msg) => format!("Errore del server: {}", msg),
        
        HttpError::Unauthorized | HttpError::Forbidden | HttpError::NotFound |
        HttpError::Http { .. } => {
            let (status, message) = match http_error {
                HttpError::Unauthorized => (401, "Unauthorized".to_string()),
                HttpError::Forbidden => (403, "Forbidden".to_string()),
                HttpError::NotFound => (404, "Not found".to_string()),
                HttpError::Http { status, message } => (*status, message.clone()),
                _ => unreachable!(),
            };
            
            // Check message content first for credential-related errors
            if message.contains("User not found") || message.contains("user not found") ||
               message.contains("Invalid credentials") || message.contains("invalid credentials") {
                "Credenziali non valide. Verifica email e password.".to_string()
            } else {
                match status {
                    401 | 403 | 404 => "Credenziali non valide. Verifica email e password.".to_string(),
                    400 => "Richiesta non valida. Verifica i dati inseriti.".to_string(),
                    429 => "Troppi tentativi di login. Attendi qualche minuto prima di riprovare.".to_string(),
                    500..=599 => "Il server sta riscontrando problemi. Riprova più tardi.".to_string(),
                    _ => "Errore durante il login. Riprova.".to_string()
                }
            }
        },
        
        _ => format!("Errore di connessione: {}", http_error),
    }
}

/// Convert HttpError to user-friendly messages for registration context
fn http_error_to_register_message(http_error: &HttpError) -> String {
    match http_error {
        HttpError::Timeout => "La richiesta ha impiegato troppo tempo. Riprova.".to_string(),
        HttpError::Network(msg) => format!("Errore di rete: {}", msg),
        HttpError::Server(msg) => format!("Errore del server: {}", msg),
        
        HttpError::Unauthorized | HttpError::Forbidden | HttpError::NotFound |
        HttpError::Http { .. } => {
            let (status, message) = match http_error {
                HttpError::Unauthorized => (401, "Unauthorized".to_string()),
                HttpError::Forbidden => (403, "Forbidden".to_string()),
                HttpError::NotFound => (404, "Not found".to_string()),
                HttpError::Http { status, message } => (*status, message.clone()),
                _ => unreachable!(),
            };
            
            match status {
                400 => "Dati non validi. Verifica le informazioni inserite.".to_string(),
                401 => "Non autorizzato. Verifica i tuoi permessi.".to_string(),
                403 => "Accesso negato. Registrazione non consentita.".to_string(),
                404 => "Servizio di registrazione non disponibile.".to_string(),
                409 => "Un utente con questa email esiste già. Usa un'altra email.".to_string(),
                422 => "Alcuni dati inseriti non sono validi. Controlla tutti i campi.".to_string(),
                429 => "Troppe richieste di registrazione. Attendi qualche minuto.".to_string(),
                500..=599 => "Il server sta riscontrando problemi. Riprova più tardi.".to_string(),
                _ => format!("Errore HTTP {}: {}", status, message),
            }
        },
        
        _ => format!("Errore di connessione: {}", http_error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::http_error::HttpError;
    use crate::error::{AuthError, StorageError};

    #[test]
    fn test_auth_error_to_login_message_invalid_credentials() {
        let error = AuthError::InvalidCredentials;
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Email o password non validi");
    }

    #[test]
    fn test_auth_error_to_login_message_invalid_input() {
        let error = AuthError::InvalidInput("Email required".to_string());
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Dati non validi: Email required");
    }

    #[test]
    fn test_auth_error_to_login_message_http_unauthorized() {
        let error = AuthError::Http(HttpError::Unauthorized);
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Credenziali non valide. Verifica email e password.");
    }

    #[test]
    fn test_auth_error_to_login_message_http_timeout() {
        let error = AuthError::Http(HttpError::Timeout);
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "La richiesta ha impiegato troppo tempo. Verifica la connessione.");
    }

    #[test]
    fn test_auth_error_to_login_message_http_status_429() {
        let error = AuthError::Http(HttpError::Http { 
            status: 429, 
            message: "Too many requests".to_string() 
        });
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Troppi tentativi di login. Attendi qualche minuto prima di riprovare.");
    }

    #[test]
    fn test_auth_error_to_login_message_http_status_404() {
        let error = AuthError::Http(HttpError::Http { 
            status: 404, 
            message: "{\"message\":\"User not found\",\"code\":404}".to_string() 
        });
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Credenziali non valide. Verifica email e password.");
    }

    #[test]
    fn test_auth_error_to_login_message_http_not_found() {
        let error = AuthError::Http(HttpError::NotFound);
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Credenziali non valide. Verifica email e password.");
    }

    #[test]
    fn test_auth_error_to_login_message_user_not_found_in_message() {
        let error = AuthError::Http(HttpError::Http { 
            status: 400, 
            message: "User not found in database".to_string() 
        });
        let message = auth_error_to_login_message(&error);
        assert_eq!(message, "Credenziali non valide. Verifica email e password.");
    }

    #[test]
    fn test_auth_error_to_register_message_invalid_input() {
        let error = AuthError::InvalidInput("Nome è obbligatorio".to_string());
        let message = auth_error_to_register_message(&error);
        assert_eq!(message, "Nome è obbligatorio");
    }

    #[test]
    fn test_auth_error_to_register_message_http_conflict() {
        let error = AuthError::Http(HttpError::Http { 
            status: 409, 
            message: "User already exists".to_string() 
        });
        let message = auth_error_to_register_message(&error);
        assert_eq!(message, "Un utente con questa email esiste già. Usa un'altra email.");
    }

    #[test]
    fn test_auth_error_to_register_message_storage_error() {
        let storage_error = StorageError::WriteError("user_profile".to_string());
        let error = AuthError::Storage(storage_error);
        let message = auth_error_to_register_message(&error);
        assert_eq!(message, "Errore nel salvataggio dei dati. Riprova.");
    }

    #[test]
    fn test_auth_error_to_register_message_validation_error() {
        let error = AuthError::ValidationError("Invalid format".to_string());
        let message = auth_error_to_register_message(&error);
        assert_eq!(message, "Errore di validazione: Invalid format");
    }

    #[test]
    fn test_auth_error_to_register_message_network_error() {
        let error = AuthError::NetworkError("Connection failed".to_string());
        let message = auth_error_to_register_message(&error);
        assert_eq!(message, "Errore di connessione: Connection failed");
    }
}
