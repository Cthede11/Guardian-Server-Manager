use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::auth::{LoginRequest, LoginResponse, RegisterRequest, UserUpdate};
use crate::core::app_state::AppState;
// use crate::core::middleware::AuthContext;
use crate::api::ApiResponse;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", axum::routing::put(update_user))
        .route("/users/:id", axum::routing::delete(delete_user))
        .route("/roles", get(get_roles))
        .route("/permissions", get(get_permissions))
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    match app_state.auth.login(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::warn!("Login failed: {}", e);
            Ok(Json(ApiResponse::<LoginResponse>::error(
                "Invalid username or password".to_string(),
            )))
        }
    }
}

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    match app_state.auth.register(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            tracing::warn!("Registration failed: {}", e);
            Ok(Json(ApiResponse::<LoginResponse>::error(
                "Registration failed".to_string(),
            )))
        }
    }
}

pub async fn logout(
    State(app_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    if let Some(token) = token {
        let _ = app_state.auth.logout(token).await;
    }
    
    Ok(Json(ApiResponse::success(())))
}

pub async fn get_current_user(
    State(app_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(Json(ApiResponse::<Value>::error(
                "Missing authorization token".to_string(),
            )));
        }
    };
    
    match app_state.auth.validate_token(token).await {
        Ok(user) => {
            let user_json = json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "role": user.role.as_str(),
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "last_login": user.last_login,
            });
            Ok(Json(ApiResponse::success(user_json)))
        }
        Err(_) => Ok(Json(ApiResponse::<Value>::error(
            "Invalid or expired token".to_string(),
        )))
    }
}

pub async fn get_users(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Value>>>, StatusCode> {
    let users = app_state.auth.get_all_users().await;
    let users_json: Vec<Value> = users
        .into_iter()
        .map(|user| {
            json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "role": user.role.as_str(),
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "last_login": user.last_login,
            })
        })
        .collect();
    
    Ok(Json(ApiResponse::success(users_json)))
}

pub async fn get_user(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(ApiResponse::<Value>::error(
                "Invalid user ID".to_string(),
            )));
        }
    };
    
    match app_state.auth.get_user(user_id).await {
        Some(user) => {
            let user_json = json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "role": user.role.as_str(),
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "last_login": user.last_login,
            });
            Ok(Json(ApiResponse::success(user_json)))
        }
        None => Ok(Json(ApiResponse::<Value>::error(
            "User not found".to_string(),
        )))
    }
}

pub async fn update_user(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(updates): Json<UserUpdate>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(ApiResponse::<Value>::error(
                "Invalid user ID".to_string(),
            )));
        }
    };
    
    match app_state.auth.update_user(user_id, updates).await {
        Ok(user) => {
            let user_json = json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "role": user.role.as_str(),
                "is_active": user.is_active,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
                "last_login": user.last_login,
            });
            Ok(Json(ApiResponse::success(user_json)))
        }
        Err(e) => Ok(Json(ApiResponse::<Value>::error(
            format!("Failed to update user: {}", e),
        )))
    }
}

pub async fn delete_user(
    State(app_state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let user_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(ApiResponse::<()>::error(
                "Invalid user ID".to_string(),
            )));
        }
    };
    
    match app_state.auth.delete_user(user_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::<()>::error(
            format!("Failed to delete user: {}", e),
        )))
    }
}

pub async fn get_roles(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Value>>>, StatusCode> {
    let roles = app_state.auth.get_roles().await;
    let roles_json: Vec<Value> = roles
        .into_iter()
        .map(|role| {
            json!({
                "id": role.id,
                "name": role.name,
                "permissions": role.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "created_at": role.created_at,
                "updated_at": role.updated_at,
            })
        })
        .collect();
    
    Ok(Json(ApiResponse::success(roles_json)))
}

pub async fn get_permissions() -> Result<Json<ApiResponse<Vec<Value>>>, StatusCode> {
    let permissions = vec![
        json!({"name": "CreateServer", "description": "Create new servers"}),
        json!({"name": "DeleteServer", "description": "Delete servers"}),
        json!({"name": "StartServer", "description": "Start servers"}),
        json!({"name": "StopServer", "description": "Stop servers"}),
        json!({"name": "RestartServer", "description": "Restart servers"}),
        json!({"name": "ViewServer", "description": "View server information"}),
        json!({"name": "EditServer", "description": "Edit server settings"}),
        json!({"name": "CreateUser", "description": "Create new users"}),
        json!({"name": "DeleteUser", "description": "Delete users"}),
        json!({"name": "EditUser", "description": "Edit user information"}),
        json!({"name": "ViewUser", "description": "View user information"}),
        json!({"name": "CreateRole", "description": "Create new roles"}),
        json!({"name": "DeleteRole", "description": "Delete roles"}),
        json!({"name": "EditRole", "description": "Edit role permissions"}),
        json!({"name": "ViewRole", "description": "View role information"}),
        json!({"name": "CreateBackup", "description": "Create server backups"}),
        json!({"name": "DeleteBackup", "description": "Delete backups"}),
        json!({"name": "RestoreBackup", "description": "Restore from backups"}),
        json!({"name": "ViewBackup", "description": "View backup information"}),
        json!({"name": "ViewLogs", "description": "View system logs"}),
        json!({"name": "ViewMetrics", "description": "View system metrics"}),
        json!({"name": "SystemSettings", "description": "Modify system settings"}),
        json!({"name": "CreateModpack", "description": "Create modpacks"}),
        json!({"name": "DeleteModpack", "description": "Delete modpacks"}),
        json!({"name": "EditModpack", "description": "Edit modpacks"}),
        json!({"name": "ViewModpack", "description": "View modpack information"}),
        json!({"name": "InstallMod", "description": "Install mods on servers"}),
        json!({"name": "UninstallMod", "description": "Uninstall mods from servers"}),
    ];
    
    Ok(Json(ApiResponse::success(permissions)))
}
