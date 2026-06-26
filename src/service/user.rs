use sea_orm::*;
use serde::Deserialize;

use crate::model::user::{self, Entity as UserEntity};

/// 创建用户请求体
#[derive(Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
}

/// 用户服务 —— 处理用户相关业务逻辑
pub struct UserService;

impl UserService {
    /// 根据 ID 查找用户
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<user::Model>, DbErr> {
        UserEntity::find_by_id(id).one(db).await
    }

    /// 获取所有用户
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<user::Model>, DbErr> {
        UserEntity::find().all(db).await
    }

    /// 创建用户
    pub async fn create(db: &DatabaseConnection, dto: CreateUserDto) -> Result<user::Model, DbErr> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let new_user = user::ActiveModel {
            name: Set(dto.name),
            email: Set(dto.email),
            created_at: Set(now),
            ..Default::default()
        };
        let result = UserEntity::insert(new_user).exec(db).await?;
        // 返回刚创建的用户
        UserEntity::find_by_id(result.last_insert_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("创建后未找到用户".to_string()))
    }
}
