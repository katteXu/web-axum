use std::{collections::HashMap, fmt::Display};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use calamine::deserialize_as_datetime_or_none;
use chrono::NaiveDateTime;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::{app_error::AuthError, constants::KEYS};

use validator_derive::Validate;

#[derive(Debug)]
pub enum TaskStatus {
    Padding(u32),
    Done,
    Err(String),
}
#[derive(Debug)]
pub struct Task {
    pub title: String,
    pub total: u32,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskBody {
    pub title: String,
    pub total: u32,
    pub status: String,
    pub progress: Option<u32>,
    pub err_msg: Option<String>,
}

impl Task {
    pub fn new(title: &str, total: usize) -> Self {
        Self {
            title: title.to_string(),
            total: total as u32,
            status: TaskStatus::Padding(0),
        }
    }
}

pub struct AppState {
    pub pool: Pool<Sqlite>,
    pub task: HashMap<Uuid, Task>,
}

/// 授权请求参数
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthPayload {
    #[validate(required)]
    pub username: Option<String>,
    #[validate(required)]
    pub password: Option<String>,
}

/// 授权响应参数
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBody {
    pub name: String,
    pub age: u8,
    pub role: String,
}

impl UserBody {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            age: 36,
            role: "administrator".to_string(),
        }
    }
}

/// 注册参数
#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
}

/// 提取器
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "username :{:?}", self.username)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub id: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    #[serde(rename(deserialize = "域名"))]
    pub domain_name: String,

    #[serde(rename(deserialize = "建站年龄"))]
    #[serde(deserialize_with = "deserialize_as_u8_or_none")]
    #[serde(default)]
    pub age: Option<u8>,

    #[serde(rename(deserialize = "记录数"))]
    #[serde(deserialize_with = "deserialize_as_u8_or_none")]
    #[serde(default)]
    pub order_no: Option<u8>,

    #[serde(rename(deserialize = "语言"))]
    pub language: Option<String>,

    #[serde(rename(deserialize = "标题"))]
    pub title: Option<String>,

    #[serde(rename(deserialize = "评分"))]
    #[serde(deserialize_with = "deserialize_as_u8_or_none")]
    #[serde(default)]
    pub score: Option<u8>,

    #[serde(rename(deserialize = "DNS"))]
    pub dns: Option<String>,

    #[serde(rename(deserialize = "注册商"))]
    pub registrar_name: Option<String>,

    #[serde(rename(deserialize = "注册商地址"))]
    pub registrar_address: Option<String>,

    #[serde(rename(deserialize = "注册人"))]
    pub registrar_by: Option<String>,

    #[serde(rename(deserialize = "Email"))]
    pub email: Option<String>,

    #[serde(rename(deserialize = "注册时间"))]
    #[serde(deserialize_with = "deserialize_as_datetime_or_none")]
    #[serde(default)]
    pub registrar_at: Option<NaiveDateTime>,

    #[serde(rename(deserialize = "到期时间"))]
    #[serde(deserialize_with = "deserialize_as_datetime_or_none")]
    #[serde(default)]
    pub expire_at: Option<NaiveDateTime>,

    #[serde(rename(deserialize = "更新时间"))]
    #[serde(deserialize_with = "deserialize_as_datetime_or_none")]
    #[serde(default)]
    pub updated_at: Option<NaiveDateTime>,

    #[serde(rename(deserialize = "备案状态"))]
    pub record_status: Option<String>,

    #[serde(rename(deserialize = "备案时间"))]
    #[serde(deserialize_with = "deserialize_as_datetime_or_none")]
    #[serde(default)]
    pub record_at: Option<NaiveDateTime>,

    #[serde(rename(deserialize = "备案主体"))]
    pub record_main_body: Option<String>,

    #[serde(rename(deserialize = "备案类型"))]
    pub record_type: Option<String>,

    #[serde(rename(deserialize = "备案号"))]
    pub record_no: Option<String>,

    #[serde(rename(deserialize = "备案名"))]
    pub record_name: Option<String>,
}

fn deserialize_as_u8_or_none<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let number = u8::deserialize(deserializer).map(Some);
    match number {
        Ok(_) => number,
        Err(_) => Ok(None),
    }
}
