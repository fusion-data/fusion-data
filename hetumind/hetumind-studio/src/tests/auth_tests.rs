#[cfg(test)]
mod tests {
    use super::*;
    use fusion_common::time::now_utc;
    use fusion_core::{application::Application, configuration::SecurityConfig, security::pwd::PwdConf};
    use fusionsql::ModelManager;
    use hetumind_context::utils::{make_token, make_refresh_token, verify_token};
    use hetumind_core::credential::TokenType;

    // Mock helper function to create a test application
    fn create_test_application() -> Application {
        // Note: This would need to be implemented based on your actual Application setup
        // For now, this is a placeholder structure
        todo!("Implement test application setup")
    }

    // Mock helper function to create test user
    fn create_test_user() -> UserForCreate {
        UserForCreate {
            email: "test@example.com".to_string(),
            phone: None,
            name: Some("Test User".to_string()),
            password: "test_password".to_string(),
            status: UserStatus::Enabled,
        }
    }

    #[tokio::test]
    async fn test_signin_success() {
        // Setup
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        let user = create_test_user();
        let user_id = UserBmc::create(&sign_svc.mm, user).await.unwrap();

        // Test signin
        let signin_req = SigninRequest {
            account: "test@example.com".to_string(),
            password: "test_password".to_string(),
        };

        let result = sign_svc.signin(signin_req).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.access_token.is_empty());
        assert!(!response.refresh_token.is_empty());
        assert_eq!(response.token_type, TokenType::Bearer);
        assert!(response.expires_in > 0);
    }

    #[tokio::test]
    async fn test_signin_invalid_credentials() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        let signin_req = SigninRequest {
            account: "nonexistent@example.com".to_string(),
            password: "wrong_password".to_string(),
        };

        let result = sign_svc.signin(signin_req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_signup_success() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        let signup_req = SignupRequest {
            email: "newuser@example.com".to_string(),
            password: "new_password".to_string(),
        };

        let result = sign_svc.signup(signup_req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_refresh_token_success() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        // First create a user and get refresh token
        let user = create_test_user();
        let user_id = UserBmc::create(&sign_svc.mm, user).await.unwrap();

        let pwd_conf = app.fusion_config().security().pwd();
        let refresh_token = make_refresh_token(user_id.to_string(), pwd_conf).unwrap();

        let refresh_req = RefreshTokenRequest { refresh_token };

        let result = sign_svc.refresh_token(refresh_req).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.access_token.is_empty());
        assert_eq!(response.token_type, TokenType::Bearer);
        assert!(response.expires_in > 0);
    }

    #[tokio::test]
    async fn test_refresh_token_invalid() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        let refresh_req = RefreshTokenRequest {
            refresh_token: "invalid_token".to_string(),
        };

        let result = sign_svc.refresh_token(refresh_req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_signout_success() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        // Create and sign in a user first
        let user = create_test_user();
        let user_id = UserBmc::create(&sign_svc.mm, user).await.unwrap();

        let pwd_conf = app.fusion_config().security().pwd();
        let access_token = make_token(user_id.to_string(), pwd_conf).unwrap();

        let signout_req = SignoutRequest {
            token: Some(access_token),
        };

        let result = sign_svc.signout(signout_req).await;
        assert!(result.is_ok());

        // Verify token is blacklisted
        let is_blacklisted = InvalidAuthTokenBmc::is_token_invalid(&sign_svc.mm, &access_token).await.unwrap();
        assert!(is_blacklisted);
    }

    #[tokio::test]
    async fn test_token_verification() {
        let app = create_test_application();
        let pwd_conf = app.fusion_config().security().pwd();

        let user_id = "test_user_123";
        let token = make_token(user_id, pwd_conf).unwrap();

        let payload = verify_token(&token, pwd_conf).unwrap();
        assert_eq!(payload.get_subject().unwrap(), user_id);
    }

    #[tokio::test]
    async fn test_token_blacklist() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        let token = "test_token_to_blacklist";
        let expires_at = now_utc() + fusion_common::time::Duration::hours(24);

        // Add token to blacklist
        let result = InvalidAuthTokenBmc::add_token(&sign_svc.mm, token, expires_at).await;
        assert!(result.is_ok());

        // Check if token is blacklisted
        let is_blacklisted = InvalidAuthTokenBmc::is_token_invalid(&sign_svc.mm, token).await.unwrap();
        assert!(is_blacklisted);
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let app = create_test_application();
        let sign_svc = SignSvc {
            mm: app.component(),
            application: app.clone(),
        };

        // Add an expired token
        let expired_token = "expired_token";
        let past_time = now_utc() - fusion_common::time::Duration::hours(1);

        InvalidAuthTokenBmc::add_token(&sign_svc.mm, expired_token, past_time).await.unwrap();

        // Add a valid token
        let valid_token = "valid_token";
        let future_time = now_utc() + fusion_common::time::Duration::hours(1);

        InvalidAuthTokenBmc::add_token(&sign_svc.mm, valid_token, future_time).await.unwrap();

        // Cleanup expired tokens
        let cleaned_count = InvalidAuthTokenBmc::cleanup_expired_tokens(&sign_svc.mm).await.unwrap();
        assert!(cleaned_count >= 1);

        // Verify expired token is gone, valid token remains
        let is_expired_blacklisted = InvalidAuthTokenBmc::is_token_invalid(&sign_svc.mm, expired_token).await.unwrap();
        let is_valid_blacklisted = InvalidAuthTokenBmc::is_token_invalid(&sign_svc.mm, valid_token).await.unwrap();

        assert!(!is_expired_blacklisted); // Should be cleaned up
        assert!(is_valid_blacklisted);    // Should still be blacklisted
    }
}