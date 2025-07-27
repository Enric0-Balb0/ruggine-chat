use utoipa::OpenApi;
use crate::{dto::{
    user_dto::{UserLoginDto, UserRegisterDto}, ApiSuccessResponseTokenReadDto, ApiSuccessResponseUserReadDto
}, entity::user::{UserStatus, UserType}};
use crate::handler::{
    auth::login_handler,
    user::{profile_handler, register_handler}
};

#[derive(OpenApi)]
#[openapi(
    paths(
        login_handler::login,
        profile_handler::profile,
        register_handler::register,
    ),
    components(
        schemas(
            UserLoginDto,
            UserRegisterDto,
            ApiSuccessResponseUserReadDto,
            ApiSuccessResponseTokenReadDto,
            UserType,
            UserStatus
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication endpoints"),
        (name = "User", description = "User management endpoints")
    ),
    info(
        title = "Ruggine Server API",
        description = "A comprehensive REST API for user management and authentication built with Rust and Axum",
        version = "0.1.0",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8002", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    )
                ),
            )
        }
    }
}
