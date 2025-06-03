use axum::{
    routing::{get, post},
    Router,
    extract::{Extension, Form},
    response::Html,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::net::TcpListener;
use axum::serve;
use dotenvy::from_filename;
use std::env;
use serde::Deserialize;

#[derive(Clone)]
struct User {
    username: String,
    password: String,
    role: String,
}

type UserStore = Arc<Mutex<HashMap<String, User>>>;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn root() -> Html<&'static str> {
    Html("Hello, world!")
}

async fn login_page() -> Html<&'static str> {
    Html(r#"
    <html>
        <body>
            <form action="/login" method="post">
                <label>Username: <input type="text" name="username" /></label><br/>
                <label>Password: <input type="password" name="password" /></label><br/>
                <button type="submit">Login</button>
            </form>
        </body>
    </html>
    "#)
}

async fn login_handler(
    Extension(user_store): Extension<UserStore>,
    Form(payload): Form<LoginForm>,
) -> Result<String, axum::http::StatusCode> {
    let users = user_store.lock().unwrap();
    if let Some(user) = users.get(&payload.username) {
        if user.password == payload.password {
            Ok(format!("Login successful! Role: {}", user.role))
        } else {
            Err(axum::http::StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

#[tokio::main]
async fn main() {
    from_filename("environment.env").ok();

    let user_store: UserStore = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut users = user_store.lock().unwrap();
        users.insert("super".into(), User { username: "super".into(), password: "pass123".into(), role: "superuser".into() });
        users.insert("admin".into(), User { username: "admin".into(), password: "adminpass".into(), role: "admin".into() });
        users.insert("worker".into(), User { username: "worker".into(), password: "workerpass".into(), role: "worker".into() });
    }

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("127.0.0.1:{}", port);

    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login_page))
        .route("/login", post(login_handler))
        .layer(Extension(user_store));

    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("Server running at http://{}", addr);

    serve(listener, app).await.unwrap();
}
