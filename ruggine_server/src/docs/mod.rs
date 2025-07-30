use utoipa::OpenApi;
use crate::{dto::{
    user_dto::{UserLoginDto, UserRegisterDto, ProfileUpdateDto}, 
    group_chat_dto::{GroupChatCreateDto, GroupChatReadDto},
    ApiSuccessResponseTokenReadDto, 
    ApiSuccessResponseUserReadDto,
    ApiSuccessResponseGroupChatReadDto
}, entity::user::{CurrentAction, Gender, UserStatus, UserType}};
use crate::handler::{
    auth_handler,
    user_handler,
    group_chat_handler
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login_handler::login,
        user_handler::profile_handler::profile,
        user_handler::update_profile_handler::update_profile,
        user_handler::register_handler::register,
        group_chat_handler::create::create,
        group_chat_handler::find_by_id::find_by_id,
    ),
    components(
        schemas(
            UserLoginDto,
            UserRegisterDto,
            ProfileUpdateDto,
            GroupChatCreateDto,
            GroupChatReadDto,
            ApiSuccessResponseUserReadDto,
            ApiSuccessResponseTokenReadDto,
            ApiSuccessResponseGroupChatReadDto,
            UserType,
            UserStatus,
            Gender,
            CurrentAction
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication endpoints"),
        (name = "User", description = "User management endpoints"),
        (name = "GroupChat", description = "Group chat management endpoints")
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
