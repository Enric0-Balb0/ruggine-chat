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
        
        // Token vicino alla scadenza (scade tra 1 ora)
        let expiring_token = TokenResponse {
            token: "expiring_token".to_string(),
            iat: now,
            exp: now + 3600, // 1 ora
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
        assert!(expiring_token.time_to_expiry() < 7200, "Expiring token should have less than 2 hours");
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
        
        // Token che scade esattamente in 2 ore
        let edge_token = TokenResponse {
            token: "edge_token".to_string(),
            iat: now,
            exp: now + 7200, // Esattamente 2 ore
        };
        
        // Token che scade in 2 ore e 1 secondo
        let just_over_token = TokenResponse {
            token: "just_over_token".to_string(),
            iat: now,
            exp: now + 7201, // 2 ore e 1 secondo
        };
        
        // Il threshold è < 7200, quindi 7200 secondi esatti non dovrebbero triggare il refresh
        assert!(!should_refresh_token(&edge_token), "Token with exactly 2 hours should not need refresh");
        assert!(!should_refresh_token(&just_over_token), "Token with slightly more than 2 hours should not need refresh");
        
        // Token che scade in 1 ora e 59 minuti (7140 secondi)
        let just_under_token = TokenResponse {
            token: "just_under_token".to_string(),
            iat: now,
            exp: now + 7140,
        };
        
        assert!(should_refresh_token(&just_under_token), "Token with less than 2 hours should need refresh");
    }
}
