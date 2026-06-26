use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 用户表实体 —— 对应数据库中的 users 表
/// SeaORM 约定：Entity + Model + ActiveModel + Column + Relation
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

/// 表关联（暂无）
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

/// ActiveModel trait 实现
impl ActiveModelBehavior for ActiveModel {}
