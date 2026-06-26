use serde::{Deserialize, Serialize};

/// 用户数据结构
#[derive(Serialize, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

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
    pub async fn find_by_id(id: u64) -> Option<User> {
        // 模拟数据库查询，后续接入真实数据层
        match id {
            1 => Some(User {
                id: 1,
                name: "Arcx".to_string(),
                email: "arcx@example.com".to_string(),
            }),
            2 => Some(User {
                id: 2,
                name: "Kira".to_string(),
                email: "kira@example.com".to_string(),
            }),
            _ => None,
        }
    }

    /// 获取所有用户
    pub async fn find_all() -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Arcx".to_string(),
                email: "arcx@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Kira".to_string(),
                email: "kira@example.com".to_string(),
            },
        ]
    }

    /// 创建用户
    pub async fn create(dto: CreateUserDto) -> User {
        // 模拟写入数据库并返回带 ID 的用户
        User {
            id: 3, // 模拟自增 ID
            name: dto.name,
            email: dto.email,
        }
    }
}
