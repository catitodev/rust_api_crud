use async_std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tide::{Request, Response, Result, StatusCode};

// Definindo nossa estrutura de dados para o usuário
// O #[derive] gera automaticamente implementações para:
// - Debug: permite imprimir a estrutura para debug
// - Clone: permite copiar a estrutura
// - Serialize: converte para JSON
// - Deserialize: cria a partir de JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

// Estrutura para dados de entrada ao criar usuário
// Não inclui id e created_at pois são gerados automaticamente
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

// Estrutura para atualização de usuário
// Todos os campos são opcionais (Option<T>)
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

// Nosso "banco de dados" em memória
// Arc (Atomic Reference Counting) permite compartilhamento entre threads
// Mutex (Mutual Exclusion) garante acesso seguro aos dados
type UserDatabase = Arc<Mutex<HashMap<String, User>>>;

// Estado compartilhado da aplicação
#[derive(Clone)]
pub struct AppState {
    pub users: UserDatabase,
}

impl AppState {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// HANDLERS - Funções que processam as requisições HTTP

// CREATE - POST /users
async fn create_user(mut req: Request<AppState>) -> Result {
    // Extrai os dados JSON do corpo da requisição
    let create_request: CreateUserRequest = req.body_json().await?;
    
    // Gera um ID único usando timestamp em nanossegundos
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let id = format!("user_{}", timestamp);
    
    // Cria o usuário com timestamp atual
    let user = User {
        id: id.clone(),
        name: create_request.name,
        email: create_request.email,
        created_at: Utc::now().to_rfc3339(),
    };
    
    // Bloqueia o mutex para acessar o HashMap de forma segura
    let mut users = req.state().users.lock().await;
    users.insert(id.clone(), user.clone());
    
    // Retorna o usuário criado com status 201 (Created)
    let mut response = Response::new(StatusCode::Created);
    response.set_body(serde_json::to_string(&user)?);
    response.set_content_type("application/json");
    Ok(response)
}

// READ ALL - GET /users
async fn get_all_users(req: Request<AppState>) -> Result {
    // Bloqueia o mutex apenas para leitura
    let users = req.state().users.lock().await;
    
    // Converte o HashMap em um vetor de usuários
    let users_list: Vec<User> = users.values().cloned().collect();
    
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(serde_json::to_string(&users_list)?);
    response.set_content_type("application/json");
    Ok(response)
}

// READ ONE - GET /users/:id
async fn get_user_by_id(req: Request<AppState>) -> Result {
    // Extrai o ID da URL
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

// UPDATE - PUT /users/:id
async fn update_user(mut req: Request<AppState>) -> Result {
    let user_id: String = req.param("id")?.to_string();
    let update_request: UpdateUserRequest = req.body_json().await?;
    
    let mut users = req.state().users.lock().await;
    
    match users.get_mut(&user_id) {
        Some(user) => {
            // Atualiza apenas os campos fornecidos
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

// DELETE - DELETE /users/:id
async fn delete_user(req: Request<AppState>) -> Result {
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

// Função principal - onde tudo começa
#[async_std::main]
async fn main() -> tide::Result<()> {
    // Inicializa o estado da aplicação
    let state = AppState::new();
    
    // Cria a aplicação Tide com nosso estado
    let mut app = tide::with_state(state);
    
    // Middleware para logs (opcional)
    app.with(tide::log::LogMiddleware::new());
    
    // Define as rotas da API
    app.at("/users").post(create_user);      // CREATE
    app.at("/users").get(get_all_users);     // READ ALL
    app.at("/users/:id").get(get_user_by_id); // READ ONE
    app.at("/users/:id").put(update_user);   // UPDATE
    app.at("/users/:id").delete(delete_user); // DELETE
    
    // Rota de health check
    app.at("/health").get(|_| async move {
        Ok(r#"{"status": "healthy"}"#)
    });
    
    println!("🚀 Servidor rodando em http://localhost:8080");
    println!("📖 Documentação das rotas:");
    println!("  POST   /users       - Criar usuário");
    println!("  GET    /users       - Listar todos os usuários");
    println!("  GET    /users/:id   - Buscar usuário por ID");
    println!("  PUT    /users/:id   - Atualizar usuário");
    println!("  DELETE /users/:id   - Deletar usuário");
    println!("  GET    /health      - Health check");
    
    // Inicia o servidor na porta 8080
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
