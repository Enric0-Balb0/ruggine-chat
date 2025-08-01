use crate::{api_success_response_alias, dto::user_dto::{UserReadDto, ProfileUpdateDto}};
use crate::dto::token_dto::TokenReadDto;
use crate::dto::group_chat_dto::{GroupChatReadDto, GroupChatCreateDto, GroupChatUpdateDto};
use crate::dto::invitation_dto::{InvitationReadDto, InvitationCreateDto, InvitationUpdateStatusDto, InvitationUpdateResponseDto};

pub mod token_dto;
pub mod user_dto;
pub mod group_chat_dto;
pub mod invitation_dto;

api_success_response_alias!(ApiSuccessResponseUserReadDto, UserReadDto);
api_success_response_alias!(ApiSuccessResponseUserUpdateDto, ProfileUpdateDto);
api_success_response_alias!(ApiSuccessResponseTokenReadDto, TokenReadDto);
api_success_response_alias!(ApiSuccessResponseGroupChatReadDto, GroupChatReadDto);
api_success_response_alias!(ApiSuccessResponseGroupChatCreateDto, GroupChatCreateDto);
api_success_response_alias!(ApiSuccessResponseGroupChatUpdateDto, GroupChatUpdateDto);
api_success_response_alias!(ApiSuccessResponseInvitationReadDto, InvitationReadDto);
api_success_response_alias!(ApiSuccessResponseInvitationUpdateDto, InvitationUpdateResponseDto);