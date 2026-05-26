use axum::{
    Extension, Json, Router,
    extract::{Request, State},
    response::IntoResponse,
    routing::{get, post},
};
use nako_api::public_client::{
    CurrentUserDto, CurrentUserResponse, LoginRequest, LoginResponse, LogoutResponse,
    UserSessionDto,
};
use nako_core::{AuthenticatedPrincipal, NakoError, UserRole, UserSessionId};

use super::error::ApiResult;
use crate::app::NakoApp;

pub(super) fn public_routes() -> Router<NakoApp> {
    Router::new().route("/auth/login", post(login))
}

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/auth/logout", post(logout))
        .route("/users/me", get(current_user))
}

async fn login(
    State(app): State<NakoApp>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let (issued, user, roles) = app
        .login_with_local_password(&request.username, &request.password)
        .await?;
    let principal = AuthenticatedPrincipal {
        user_id: user.id,
        principal_id: user.principal_id,
        roles,
        bootstrap: false,
    };

    Ok(Json(LoginResponse {
        session: UserSessionDto {
            token: issued.token,
            expires_at_ms: issued.session.expires_at_ms,
        },
        account: CurrentUserResponse {
            user: current_user_dto(&user.username, &user.display_name, &principal),
        },
    }))
}

async fn current_user(
    Extension(principal): Extension<AuthenticatedPrincipal>,
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    let user = app
        .get_user(principal.user_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "user",
            id: principal.user_id.to_string(),
        })?;

    Ok(Json(CurrentUserResponse {
        user: current_user_dto(&user.username, &user.display_name, &principal),
    }))
}

async fn logout(State(app): State<NakoApp>, request: Request) -> ApiResult<impl IntoResponse> {
    let session_id = request.extensions().get::<UserSessionId>().copied();
    let revoked = if let Some(session_id) = session_id {
        app.revoke_user_session(session_id).await?
    } else {
        false
    };

    Ok(Json(LogoutResponse { revoked }))
}

fn current_user_dto(
    username: &str,
    display_name: &str,
    principal: &AuthenticatedPrincipal,
) -> CurrentUserDto {
    CurrentUserDto {
        id: principal.user_id.to_string(),
        username: username.to_owned(),
        display_name: display_name.to_owned(),
        roles: principal
            .roles
            .iter()
            .map(|role| role_to_dto(*role).to_owned())
            .collect(),
        bootstrap: principal.bootstrap,
    }
}

fn role_to_dto(role: UserRole) -> &'static str {
    role.as_str()
}
