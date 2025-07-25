use async_std::sync::{Arc, Mutex};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tide::{Request, Response, Result, StatusCode};

// Estruturas de dados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

// Estruturas para autenticação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

// Estado da aplicação
type UserDatabase = Arc<Mutex<HashMap<String, User>>>;
type AdminDatabase = Arc<Mutex<HashMap<String, Admin>>>;

#[derive(Clone)]
pub struct AppState {
    pub users: UserDatabase,
    pub admins: AdminDatabase,
    pub jwt_secret: String,
}

impl AppState {
    fn new() -> Self {
        let mut admins = HashMap::new();
        
        // Criar admin padrão (usuário: admin, senha: admin123)
        let password_hash = hash("admin123", DEFAULT_COST).unwrap();
        let admin = Admin {
            username: "admin".to_string(),
            password_hash,
            created_at: Utc::now().to_rfc3339(),
        };
        admins.insert("admin".to_string(), admin);

        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            admins: Arc::new(Mutex::new(admins)),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
        }
    }
}

// Middleware de autenticação
async fn extract_token_claims(req: &Request<AppState>) -> Option<Claims> {
    let auth_header = req.header("Authorization")?;
    let token = auth_header.as_str().strip_prefix("Bearer ")?;
    
    let jwt_secret = &req.state().jwt_secret;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ).ok()?;
    
    Some(token_data.claims)
}

// ROTAS DE AUTENTICAÇÃO

// POST /auth/login - Fazer login e receber token
async fn login(mut req: Request<AppState>) -> Result {
    let login_request: LoginRequest = req.body_json().await?;
    let admins = req.state().admins.lock().await;
    
    if let Some(admin) = admins.get(&login_request.username) {
        if verify(&login_request.password, &admin.password_hash).unwrap_or(false) {
            // Gerar JWT token
            let expiration = Utc::now() + Duration::hours(24);
            let claims = Claims {
                username: admin.username.clone(),
                exp: expiration.timestamp() as usize,
            };
            
            let jwt_secret = &req.state().jwt_secret;
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(jwt_secret.as_ref()),
            ).map_err(|_| tide::Error::from_str(500, "Failed to generate token"))?;
            
            let response_data = LoginResponse {
                token,
                expires_in: expiration.to_rfc3339(),
            };
            
            let mut response = Response::new(StatusCode::Ok);
            response.set_body(serde_json::to_string(&response_data)?);
            response.set_content_type("application/json");
            return Ok(response);
        }
    }
    
    let mut response = Response::new(StatusCode::Unauthorized);
    response.set_body(r#"{"error": "Invalid credentials"}"#);
    response.set_content_type("application/json");
    Ok(response)
}

// GET /auth/verify - Verificar se token é válido
async fn verify_token(req: Request<AppState>) -> Result {
    match extract_token_claims(&req).await {
        Some(claims) => {
            let mut response = Response::new(StatusCode::Ok);
            response.set_body(serde_json::to_string(&claims)?);
            response.set_content_type("application/json");
            Ok(response)
        }
        None => {
            let mut response = Response::new(StatusCode::Unauthorized);
            response.set_body(r#"{"error": "Invalid or expired token"}"#);
            response.set_content_type("application/json");
            Ok(response)
        }
    }
}

// ROTAS PROTEGIDAS (necessitam autenticação)

// CREATE - POST /users (PROTEGIDA)
async fn create_user(mut req: Request<AppState>) -> Result {
    // Verificar autenticação
    if extract_token_claims(&req).await.is_none() {
        let mut response = Response::new(StatusCode::Unauthorized);
        response.set_body(r#"{"error": "Authentication required for this operation"}"#);
        response.set_content_type("application/json");
        return Ok(response);
    }
    
    let create_request: CreateUserRequest = req.body_json().await?;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let id = format!("user_{}", timestamp);
    
    let user = User {
        id: id.clone(),
        name: create_request.name,
        email: create_request.email,
        created_at: Utc::now().to_rfc3339(),
    };
    
    let mut users = req.state().users.lock().await;
    users.insert(id.clone(), user.clone());
    
    let mut response = Response::new(StatusCode::Created);
    response.set_body(serde_json::to_string(&user)?);
    response.set_content_type("application/json");
    Ok(response)
}

// UPDATE - PUT /users/:id (PROTEGIDA)
async fn update_user(mut req: Request<AppState>) -> Result {
    // Verificar autenticação
    if extract_token_claims(&req).await.is_none() {
        let mut response = Response::new(StatusCode::Unauthorized);
        response.set_body(r#"{"error": "Authentication required for this operation"}"#);
        response.set_content_type("application/json");
        return Ok(response);
    }
    
    let user_id: String = req.param("id")?.to_string();
    let update_request: UpdateUserRequest = req.body_json().await?;
    
    let mut users = req.state().users.lock().await;
    
    match users.get_mut(&user_id) {
        Some(user) => {
            if let Some(name) = update_request.name {
                user.name = name;
            }
            if let Some(email) = update_request.email {
                user.email = email;
            }
            
            let mut response = Response::new(StatusCode::Ok);
            response.set_body(serde_json::to_string(user)?);
            response.set_content_type("application/json");
            Ok(response)
        }
        None => {
            let mut response = Response::new(StatusCode::NotFound);
            response.set_body(r#"{"error": "User not found"}"#);
            response.set_content_type("application/json");
            Ok(response)
        }
    }
}

// DELETE - DELETE /users/:id (PROTEGIDA)
async fn delete_user(req: Request<AppState>) -> Result {
    // Verificar autenticação
    if extract_token_claims(&req).await.is_none() {
        let mut response = Response::new(StatusCode::Unauthorized);
        response.set_body(r#"{"error": "Authentication required for this operation"}"#);
        response.set_content_type("application/json");
        return Ok(response);
    }
    
    let user_id: String = req.param("id")?.to_string();
    
    let mut users = req.state().users.lock().await;
    
    match users.remove(&user_id) {
        Some(_) => {
            let mut response = Response::new(StatusCode::Ok);
            response.set_body(r#"{"message": "User deleted successfully"}"#);
            response.set_content_type("application/json");
            Ok(response)
        }
        None => {
            let mut response = Response::new(StatusCode::NotFound);
            response.set_body(r#"{"error": "User not found"}"#);
            response.set_content_type("application/json");
            Ok(response)
        }
    }
}

// ROTAS PÚBLICAS (não necessitam autenticação)

// READ ALL - GET /users (PÚBLICA)
async fn get_all_users(req: Request<AppState>) -> Result {
    let users = req.state().users.lock().await;
    let users_list: Vec<User> = users.values().cloned().collect();
    
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(serde_json::to_string(&users_list)?);
    response.set_content_type("application/json");
    Ok(response)
}

// READ ONE - GET /users/:id (PÚBLICA)
async fn get_user_by_id(req: Request<AppState>) -> Result {
    let user_id: String = req.param("id")?.to_string();
    let users = req.state().users.lock().await;
    
    match users.get(&user_id) {
        Some(user) => {
            let mut response = Response::new(StatusCode::Ok);
            response.set_body(serde_json::to_string(user)?);
            response.set_content_type("application/json");
            Ok(response)
        }
        None => {
            let mut response = Response::new(StatusCode::NotFound);
            response.set_body(r#"{"error": "User not found"}"#);
            response.set_content_type("application/json");
            Ok(response)
        }
    }
}

// Função principal
#[async_std::main]
async fn main() -> tide::Result<()> {
    // Carregar variáveis de ambiente se existir arquivo .env
    dotenv::dotenv().ok();
    
    let state = AppState::new();
    let mut app = tide::with_state(state);
    
    // Middleware para logs
    app.with(tide::log::LogMiddleware::new());
    
    // ROTAS DE AUTENTICAÇÃO
    app.at("/auth/login").post(login);
    app.at("/auth/verify").get(verify_token);
    
    // ROTAS PROTEGIDAS (necessitam token JWT)
    app.at("/users").post(create_user);           // CREATE (protegida)
    app.at("/users/:id").put(update_user);        // UPDATE (protegida)
    app.at("/users/:id").delete(delete_user);     // DELETE (protegida)
    
    // ROTAS PÚBLICAS (sem autenticação)
    app.at("/users").get(get_all_users);          // READ ALL (pública)
    app.at("/users/:id").get(get_user_by_id);     // READ ONE (pública)
    
    // Health check
    app.at("/health").get(|_| async move {
        Ok(r#"{"status": "healthy", "auth": "enabled"}"#)
    });
    
    // Usar a porta definida pelo Railway ou 8080 localmente
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);
    
    println!("🚀 Servidor rodando em {}", address);
    println!("🔐 Autenticação JWT habilitada");
    println!("📖 Documentação das rotas:");
    println!("  POST   /auth/login     - Fazer login (receber token)");
    println!("  GET    /auth/verify    - Verificar token");
    println!("  📖 ROTAS PÚBLICAS:");
    println!("  GET    /users          - Listar usuários (sem auth)");
    println!("  GET    /users/:id      - Buscar usuário (sem auth)");
    println!("  🔒 ROTAS PROTEGIDAS (necessitam Bearer token):");
    println!("  POST   /users          - Criar usuário");
    println!("  PUT    /users/:id      - Atualizar usuário");
    println!("  DELETE /users/:id      - Deletar usuário");
    println!("  GET    /health         - Health check");
    println!("");
    println!("👤 Admin padrão: username=admin, password=admin123");
    println!("🔑 Para acessar rotas protegidas:");
    println!("   1. POST /auth/login com {{\"username\":\"admin\",\"password\":\"admin123\"}}");
    println!("   2. Use o token retornado: Authorization: Bearer <token>");
    
    app.listen(&address).await?;
    Ok(())
}
