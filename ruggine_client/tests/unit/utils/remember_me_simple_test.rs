use ruggine_client_ui::types::TokenResponse;
use ruggine_client_ui::hooks::should_refresh_token;

#[cfg(test)]
mod remember_me_simple_tests {
    use super::*;

    #[test]
    fn test_token_expiry_logic() {
        let now = chrono::Utc::now().timestamp();
        
        // Token valido (scade tra 4 ore)
        let valid_token = TokenResponse {
            token: "valid_token".to_string(),
            iat: now,
            exp: now + 14400, // 4 ore
        };
        
        // Token vicino alla scadenza (scade tra 10 minuti - dovrebbe essere refreshato)
        // Per un token di 1 ora, deve rimanere meno del 25% = 15 min per essere refreshato
        let expiring_token = TokenResponse {
            token: "expiring_token".to_string(),
            iat: now - 3000, // Creato 50 min fa
            exp: now + 600,  // Scade tra 10 minuti
        };
        
        // Token scaduto
        let expired_token = TokenResponse {
            token: "expired_token".to_string(),
            iat: now - 7200,
            exp: now - 3600, // Scaduto 1 ora fa
        };
        
        // Test metodi TokenResponse
        assert!(!valid_token.is_expired(), "Valid token should not be expired");
        assert!(!expiring_token.is_expired(), "Expiring token should not be expired yet");
        assert!(expired_token.is_expired(), "Expired token should be expired");
        
        assert!(valid_token.time_to_expiry() > 7200, "Valid token should have more than 2 hours");
        assert!(expiring_token.time_to_expiry() < 900, "Expiring token should have less than 15 minutes");
        assert_eq!(expired_token.time_to_expiry(), 0, "Expired token should have 0 time left");
        
        // Test logica should_refresh_token
        assert!(!should_refresh_token(&valid_token), "Valid token should not need refresh");
        assert!(should_refresh_token(&expiring_token), "Expiring token should need refresh");
        assert!(should_refresh_token(&expired_token), "Expired token should need refresh");
    }
    
    #[test]
    fn test_short_lived_tokens() {
        let now = chrono::Utc::now().timestamp();
        
        // Token di 30 minuti (1800 secondi)
        let short_token_fresh = TokenResponse {
            token: "short_fresh".to_string(),
            iat: now,
            exp: now + 1800, // 30 minuti
        };
        
        // Token di 30 minuti con 5 minuti rimanenti (dovrebbe essere refreshato)
        let short_token_expiring = TokenResponse {
            token: "short_expiring".to_string(),
            iat: now - 1500, // Creato 25 min fa
            exp: now + 300,   // Scade tra 5 minuti
        };
        
        // Token di 15 minuti con 3 minuti rimanenti
        let very_short_token = TokenResponse {
            token: "very_short".to_string(),
            iat: now - 720,  // Creato 12 min fa
            exp: now + 180,  // Scade tra 3 minuti
        };
        
        // Test: token corti dovrebbero avere threshold più aggressivi
        assert!(!should_refresh_token(&short_token_fresh), "Fresh short token should not need refresh");
        assert!(should_refresh_token(&short_token_expiring), "Short token with 5 min left should need refresh");
        assert!(should_refresh_token(&very_short_token), "Very short token with 3 min left should need refresh");
    }
    
    #[test]
    fn test_refresh_threshold_edge_cases() {
        let now = chrono::Utc::now().timestamp();
        
        // Test con token di durata lunga (>2 ore)
        // Per token > 2 ore: usa la logica originale (refresh quando rimangono < 2 ore)
        let long_token = TokenResponse {
            token: "long_token".to_string(),
            iat: now,
            exp: now + 10800, // 3 ore totali
        };
        
        // Token lungo con 1 ora e 50 minuti rimanenti (< 2 ore)
        let long_token_should_refresh = TokenResponse {
            token: "long_should_refresh".to_string(),
            iat: now - 4200, // Creato 70 minuti fa  
            exp: now + 6600, // Scade in 110 minuti (1h 50m)
        };
        
        assert!(!should_refresh_token(&long_token), "Fresh long token should not need refresh");
        assert!(should_refresh_token(&long_token_should_refresh), "Long token with <2h remaining should need refresh");
        
        // Test con token di durata media (esattamente 2 ore)
        // Per token <= 2 ore: refresh quando rimane meno del 33%
        let medium_token = TokenResponse {
            token: "medium_token".to_string(),
            iat: now,
            exp: now + 7200, // Esattamente 2 ore
        };
        
        // Token medio con 30 minuti rimanenti (< 33% di 2 ore = 40 minuti)
        let medium_token_should_refresh = TokenResponse {
            token: "medium_should_refresh".to_string(),
            iat: now - 5400, // Creato 90 minuti fa
            exp: now + 1800,  // Scade in 30 minuti
        };
        
        assert!(!should_refresh_token(&medium_token), "Fresh medium token should not need refresh");
        assert!(should_refresh_token(&medium_token_should_refresh), "Medium token with <33% time remaining should need refresh");
    }
}
