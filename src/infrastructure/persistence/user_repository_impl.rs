use crate::{
    domain::repositories::{User, UserRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use chrono::Utc;
use entity::{UserActiveModel, UserColumn, UserEntity, UserModel};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};
use uuid::Uuid;

pub struct SeaOrmUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: UserModel) -> User {
        User {
            id: model.id,
            email: model.email,
            password: model.password,
            first_name: model.first_name,
            last_name: model.last_name,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn to_active_model(user: User) -> UserActiveModel {
        UserActiveModel {
            id: Set(user.id),
            email: Set(user.email),
            password: Set(user.password),
            first_name: Set(user.first_name),
            last_name: Set(user.last_name),
            is_active: Set(user.is_active),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn create(&self, user: User) -> Result<User, ApplicationError> {
        let active_model = Self::to_active_model(user);

        let result = UserEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError> {
        let result = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.map(Self::to_domain))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        let result = UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.map(Self::to_domain))
    }

    async fn exists_by_id(&self, id: Uuid) -> Result<bool, ApplicationError> {
        let count = UserEntity::find()
            .filter(UserColumn::Id.eq(id))
            .count(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError> {
        let count = UserEntity::find()
            .filter(UserColumn::Email.eq(email.to_string()))
            .count(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn activate(&self, id: Uuid) -> Result<User, ApplicationError> {
        let user = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "User with the given ID not found".to_string(),
            })?;

        let mut active_model: UserActiveModel = user.into();
        active_model.is_active = Set(true);
        active_model.updated_at = Set(Utc::now().fixed_offset());

        let result = UserEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn update_password(
        &self,
        id: Uuid,
        new_password: &str,
    ) -> Result<User, ApplicationError> {
        let user = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "User with the given ID not found".to_string(),
            })?;

        let mut active_model: UserActiveModel = user.into();
        active_model.password = Set(new_password.to_string());
        active_model.updated_at = Set(Utc::now().fixed_offset());

        let result = UserEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }
}
