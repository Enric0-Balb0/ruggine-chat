use crate::{api_success_response_alias, dto::user_dto::UserReadDto};
use crate::dto::token_dto::TokenReadDto;

pub mod token_dto;
pub mod user_dto;

api_success_response_alias!(ApiSuccessResponseUserReadDto, UserReadDto);
api_success_response_alias!(ApiSuccessResponseTokenReadDto, TokenReadDto);