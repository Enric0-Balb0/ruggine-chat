use utoipa::OpenApi;
use crate::{dto::{
        user_dto::{UserLoginDto, UserRegisterDto, ProfileUpdateDto}, 
        group_chat_dto::{GroupChatCreateDto},
        group_membership_dto::{LeaveGroupMembershipDto},
        ApiSuccessResponseTokenReadDto, 
        ApiSuccessResponseUserReadDto,
        ApiSuccessResponseUserOptionReadDto,
        ApiSuccessResponseGroupChatReadDto,
        invitation_dto::{
            InvitationCreateDto,
            InvitationUpdateStatusDto
        },
        ApiSuccessResponseInvitationReadDto,
        ApiSuccessResponseInvitationUpdateDto,
        ApiSuccessResponseGroupMembershipReadDto,
        ApiSuccessResponseVecGroupMembershipReadDto,
        ApiSuccessResponseInvitationUpdateResponseDto,
        ApiSuccessResponseVecInvitationReadDto,
        ApiSuccessResponseTextMessageReadDto,
        PaginatedTextMessageResponse,
        text_message_pagination_dto::{
            FindTextMessagesByGroupQuery,
            TextMessagePaginationQuery,
        },
        text_message_dto::{
            TextMessageCreateDto,
            TextMessageReadDto,
        },
    },
    entity::{user::{Gender, UserStatus, UserType}, invitation::{InvitationStatus}, group_membership::{MembershipStatus, MemberRole, CurrentAction}},
};
use crate::handler::{
    auth_handler,
    user_handler,
    group_chat_handler,
    invitation_handler,
    group_membership_handler,
    text_message_handler
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login_handler::login,
        user_handler::profile_handler::profile,
        user_handler::update_profile_handler::update_profile,
        user_handler::register_handler::register,
        user_handler::find_by_id_handler::find_by_id,
        user_handler::find_by_username_handler::find_by_username,
        group_chat_handler::create::create,
        group_chat_handler::find_by_id::find_by_id,
        invitation_handler::send::send,
        invitation_handler::find_by_id_and_user_id::find_by_id_and_user_id,
        invitation_handler::update_status::update_status,
        invitation_handler::find_by_user_id::find_by_user_id,
        group_membership_handler::find_by_group_chat_id::find_by_group_chat_id,
        group_membership_handler::find_by_user_id::find_by_user_id,
        group_membership_handler::find_active_by_id_and_user_id::find_active_by_id_and_user_id,
        group_membership_handler::leave_group::leave_group,
        text_message_handler::find_by_group_chat_id::find_by_group_chat_id,
        text_message_handler::create::create
    ),
    components(
        schemas(
            UserLoginDto,
            UserRegisterDto,
            ProfileUpdateDto,
            GroupChatCreateDto,
            LeaveGroupMembershipDto,
            ApiSuccessResponseUserReadDto,
            ApiSuccessResponseUserOptionReadDto,
            ApiSuccessResponseTokenReadDto,
            ApiSuccessResponseGroupChatReadDto,
            UserType,
            UserStatus,
            Gender,
            CurrentAction,
            ApiSuccessResponseInvitationReadDto,
            InvitationCreateDto,
            InvitationUpdateStatusDto,
            ApiSuccessResponseInvitationUpdateDto,
            InvitationStatus,
            MembershipStatus,
            MemberRole,
            ApiSuccessResponseGroupMembershipReadDto,
            ApiSuccessResponseVecGroupMembershipReadDto,
            ApiSuccessResponseInvitationUpdateResponseDto,
            ApiSuccessResponseVecInvitationReadDto,
            PaginatedTextMessageResponse,
            FindTextMessagesByGroupQuery,
            TextMessagePaginationQuery,
            TextMessageCreateDto,
            TextMessageReadDto,
            ApiSuccessResponseTextMessageReadDto
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication endpoints"),
        (name = "User", description = "User management endpoints"),
        (name = "GroupChat", description = "Group chat management endpoints"),
        (name = "Invitation", description = "Invitation management endpoints"),
        (name = "GroupMembership", description = "Group membership management endpoints"),
        (name = "TextMessage", description = "Text message management endpoints"),
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
