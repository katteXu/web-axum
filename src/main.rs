use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{encode, Header};
use rand_core::OsRng;
use rust_axum_with_vim::{
    app_error::{AppError, AuthError},
    constants::{DATABASE_URL, KEYS, PORT},
    file::excel_to_record,
    model::{
        AppState, AuthBody, AuthPayload, Claims, Record, RegisterPayload, Task, TaskBody,
        TaskStatus, UserBody, UserModel,
    },
};
use std::{borrow::BorrowMut, collections::HashMap, path::PathBuf, sync::Arc};
use validator::Validate;

use serde_json::{json, Value};
use sqlx::{Pool, Sqlite, SqlitePool};
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener, sync::Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_axum_with_vim=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载环境变量
    dotenv::dotenv().ok();

    // 连接数据库
    let pool = SqlitePool::connect(&DATABASE_URL).await?;

    let app_state = Arc::new(Mutex::new(AppState {
        pool,
        task: HashMap::new(),
    }));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/login", post(login_handler))
        .route("/api/register", post(register_handler))
        .route("/api/upload", post(upload_handler))
        .route("/api/user/:id", get(get_user_handler))
        .route("/api/task/:task_id", get(show_task_handler))
        .with_state(Arc::clone(&app_state))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    let url = format!("127.0.0.1:{}", &PORT.to_string());

    let listener = TcpListener::bind(&url).await.unwrap();

    tracing::debug!("listen at {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index_handler() -> Result<(), AppError> {
    Ok(())
}

// 获取用户信息
async fn get_user_handler(_claims: Claims) -> Result<Json<UserBody>, AppError> {
    let user = UserBody::new("katte");

    if &user.name == "katte" {}

    Ok(Json(user))
}

/// 注册用户
async fn register_handler(
    State(app_state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<RegisterPayload>,
) -> Result<Json<Value>, AppError> {
    let pool = &app_state.lock().await.pool;
    let id = Uuid::new_v4().to_string();

    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user WHERE username = ?)")
            .bind(&body.username)
            .fetch_one(pool)
            .await?;

    if let Some(exists) = user_exists {
        if exists {
            return Err(AppError(anyhow::anyhow!("用户名已存在")));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash_password = Argon2::default()
        .hash_password(&body.password.as_bytes(), &salt)
        .map_err(|_| anyhow::anyhow!("hash password error"))?
        .to_string();

    let _result = sqlx::query("INSERT INTO user (id,username,password) values(?,?,?)")
        .bind(id)
        .bind(&body.username)
        .bind(hash_password)
        .execute(pool)
        .await?;

    let response = json!({
        "status":"success",
        "message":"注册成功",
    });

    Ok(Json(response))
}

/// 登录
async fn login_handler(
    State(app_state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    body.validate().map_err(|_e| {
        return AuthError::MissingCredentials;
    })?;

    let username = body.username.unwrap();
    let password = body.password.unwrap();
    let pool = &app_state.lock().await.pool;

    let user = sqlx::query_as!(
        UserModel,
        "select id, username, password from user where username=?",
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthError::WrongCredentials)?;

    if user.is_none() {
        return Err(AuthError::WrongCredentials);
    }

    let user = user.unwrap();

    verify_password(password, &user.password).map_err(|_| AuthError::WrongCredentials)?;

    let claims = Claims {
        username,
        exp: 2000000000,
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthBody::new(token)))
}

/// 上传数据
async fn upload_handler(
    State(app_state): State<Arc<Mutex<AppState>>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // let pool = &app_state.lock().await.pool;

    while let Some(field) = multipart.next_field().await? {
        let file_name = field.file_name().unwrap().to_string();
        let file_data = field.bytes().await.unwrap();

        let path = format!("./upload/{}", &file_name);
        let file_path = PathBuf::from(&path);

        let mut file = File::create(&file_path).await?;

        file.write_all(&file_data).await?;

        // excel 转 record
        let data = excel_to_record(&file_path)?;

        // 创建任务
        let task = Task::new("导入数据", data.len());
        let id = build_task(Arc::clone(&app_state), task).await;
        let app_state = Arc::clone(&app_state);

        // 导入数据库
        tokio::spawn(async move {
            let mut data_iter = data.iter();
            while let Some(record) = data_iter.next() {
                let mut state = app_state.lock().await;

                // 插入
                insert_excel_record(&state.pool, record).await.unwrap();

                // 标记任务进度
                let task = state.task.get_mut(&id).unwrap();
                match task.status {
                    TaskStatus::Padding(num) => {
                        let next = num + 1;
                        if next >= task.total {
                            task.status = TaskStatus::Done;
                            break;
                        } else {
                            task.status = TaskStatus::Padding(next);
                        }
                    }
                    TaskStatus::Done => {}
                    TaskStatus::Err(ref e) => println!("{:?}", e),
                }
            }
        });

        let response = json!({
            "status": "success",
            "message": "上传成功",
            "task_id": id
        });

        return Ok(Json(response));
    }

    let response = json!({
        "status":"fail",
        "message":"上传失败"
    });

    Ok(Json(response))
}

/// 查看任务进度
async fn show_task_handler(
    State(app_state): State<Arc<Mutex<AppState>>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskBody>, AppError> {
    let taks_map = &app_state.lock().await.task;

    let task = taks_map.get(&task_id).unwrap();
    let mut progress = None;
    let mut err_msg = None;
    let status = match &task.status {
        TaskStatus::Done => "done",
        TaskStatus::Padding(n) => {
            progress = Some(*n);
            "padding"
        }
        TaskStatus::Err(e) => {
            err_msg = Some(e.to_string());
            "error"
        }
    };
    let response = TaskBody {
        title: task.title.to_string(),
        status: status.to_string(),
        progress,
        total: task.total,
        err_msg,
    };
    Ok(Json(response))
}

/// 密码验证
fn verify_password(password: String, password_hash: &String) -> Result<(), String> {
    let hash =
        PasswordHash::new(&password_hash).map_err(|e| format!("invalid password hash: {}", e))?;

    hash.verify_password(&[&Argon2::default()], password)
        .map_err(|e| match e {
            argon2::password_hash::Error::Password => "invalid password".to_string(),
            _ => format!("falied to verify password hash: {}", e),
        })
}

/// 创建任务
async fn build_task(app_state: Arc<Mutex<AppState>>, task: Task) -> Uuid {
    let id = Uuid::new_v4();
    app_state.lock().await.task.borrow_mut().insert(id, task);
    id
}

async fn insert_excel_record(pool: &Pool<Sqlite>, record: &Record) -> Result<(), anyhow::Error> {
    let id = Uuid::new_v4().to_string();

    let _result = sqlx::query!(
        r#"
            INSERT INTO domain
            (
            id, domain_name, domain_age, order_no, language,
            title, score, dns, registrar_name, registrar_address,
            registrar_by, registrar_at, email, expire_at,
            record_status, record_at, record_main_body, record_type, record_no,
            record_name
            ) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
        "#,
        id,
        record.domain_name,
        record.age,
        record.order_no,
        record.language,
        record.title,
        record.score,
        record.dns,
        record.registrar_name,
        record.registrar_address,
        record.registrar_by,
        record.registrar_at,
        record.email,
        record.expire_at,
        record.record_status,
        record.record_at,
        record.record_main_body,
        record.record_type,
        record.record_no,
        record.record_name
    )
    .execute(pool)
    .await?;

    Ok(())
}
