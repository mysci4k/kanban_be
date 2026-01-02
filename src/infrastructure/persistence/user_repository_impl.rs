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
use tracing::{debug, error, instrument};
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
    #[instrument(
        name = "db.user.create",
        skip(self, user),
        fields(
            db.operation = "insert",
            user.id = %user.id
        ),
        err
    )]
    async fn create(&self, user: User) -> Result<User, ApplicationError> {
        debug!("Creating user");
        let active_model = Self::to_active_model(user);

        let result = UserEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to create user");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.user.find_by_id",
        skip(self, id),
        fields(
            db.operation = "select",
            user.id = %id
        ),
        err
    )]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError> {
        debug!("Finding user by ID");
        let result = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find user by ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.map(Self::to_domain))
    }

    #[instrument(
        name = "db.user.find_by_email",
        skip(self, email),
        fields(
            db.operation = "select",
            user.email = %email
        ),
        err
    )]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError> {
        debug!("Finding user by email");
        let result = UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find user by email");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.map(Self::to_domain))
    }

    #[instrument(
        name = "db.user.exists_by_id",
        skip(self, id),
        fields(
            db.operation = "count",
            user.id = %id
        ),
        err
    )]
    async fn exists_by_id(&self, id: Uuid) -> Result<bool, ApplicationError> {
        debug!("Checking if user exists by ID");
        let count = UserEntity::find()
            .filter(UserColumn::Id.eq(id))
            .count(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to check if user exists by ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(count > 0)
    }

    #[instrument(
        name = "db.user.exists_by_email",
        skip(self, email),
        fields(
            db.operation = "count",
            user.email = %email
        ),
        err
    )]
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError> {
        debug!("Checking if user exists by email");
        let count = UserEntity::find()
            .filter(UserColumn::Email.eq(email.to_string()))
            .count(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to check if user exists by email");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(count > 0)
    }

    #[instrument(
        name = "db.user.activate",
        skip(self, id),
        fields(
            db.operation = "update",
            user.id = %id
        ),
        err
    )]
    async fn activate(&self, id: Uuid) -> Result<User, ApplicationError> {
        debug!("Activating user account");
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
            .map_err(|err| {
                error!(error = %err, "Failed to activate user account");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.user.update_password",
        skip(self, id, new_password),
        fields(
            db.operation = "update",
            user.id = %id
        ),
        err
    )]
    async fn update_password(
        &self,
        id: Uuid,
        new_password: &str,
    ) -> Result<User, ApplicationError> {
        debug!("Updating user password");
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
            .map_err(|err| {
                error!(error = %err, "Failed to update user password");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }
}
