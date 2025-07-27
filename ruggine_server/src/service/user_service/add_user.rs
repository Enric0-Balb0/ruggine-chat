use crate::dto::user_dto::UserRegisterDto;
use crate::entity::user::{NewUser, User};
use crate::service::user_service::UserService;
use sqlx::Error;

impl UserService {
    pub async fn add_user(&self, payload: UserRegisterDto) -> Result<User, Error> {
        let hashed_password = bcrypt::hash(payload.password, 4).unwrap();
        let new_user = NewUser {
            first_name: payload.first_name,
            last_name: payload.last_name,
            username: payload.username,
            email: payload.email,
            password: hashed_password,
            user_status: Default::default(),
            user_type: Default::default(),
            birthday: payload.birthday,
            address: payload.address,
            gender: payload.gender,
        };
        let user_id = self.user_repo.insert(new_user).await?;
        let user = self.user_repo.find(user_id).await?;
        Ok(user)
    }
}
