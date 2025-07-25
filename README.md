# 🦀 Rust API CRUD

Uma API REST completa desenvolvida em Rust utilizando tecnologias modernas para alta performance e segurança de memória. Este projeto demonstra conceitos avançados como programação assíncrona, gerenciamento de estado thread-safe e arquitetura de microserviços.

## 🚀 Tecnologias Utilizadas

- **[Rust](https://www.rust-lang.org/)** - Linguagem de programação de sistemas com foco em segurança e performance
- **[async-std](https://async.rs/)** - Runtime assíncrono para operações não-bloqueantes
- **[Tide](https://github.com/http-rs/tide)** - Framework web moderno e ergonômico
- **[Serde](https://serde.rs/)** - Framework de serialização/deserialização para Rust
- **[Arc/Mutex](https://doc.rust-lang.org/std/sync/)** - Primitivas de sincronização para compartilhamento seguro de dados

## 📋 Funcionalidades

### Operações CRUD Completas
- ✅ **CREATE** - Criar novos usuários
- ✅ **READ** - Listar todos os usuários ou buscar por ID
- ✅ **UPDATE** - Atualizar dados de usuários existentes
- ✅ **DELETE** - Remover usuários do sistema

### Recursos Avançados
- 🔄 **Programação Assíncrona** - Suporte a milhares de conexões simultâneas
- 🛡️ **Memory Safety** - Gerenciamento automático e seguro de memória
- 🧵 **Thread Safety** - Compartilhamento seguro de dados entre threads
- 📊 **Serialização JSON** - Conversão automática entre structs Rust e JSON
- ⚡ **Alta Performance** - Zero-cost abstractions e otimizações de baixo nível

## 🏗️ Arquitetura do Projeto

```
src/
├── main.rs              # Ponto de entrada e configuração do servidor
├── models/              # Estruturas de dados (User, Requests)
├── handlers/            # Manipuladores de rotas HTTP
├── state/               # Gerenciamento de estado da aplicação
└── utils/               # Utilitários e helpers
```

### Modelo de Dados

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}
```

## 🔧 Instalação e Configuração

### Pré-requisitos

- Rust 1.70+ instalado
- Cargo (geralmente vem com Rust)

### Clonagem e Execução

```bash
# Clone o repositório
git clone https://github.com/catitodev/rust_api_crud.git
cd rust_api_crud

# Instale as dependências e compile
cargo build

# Execute o servidor
cargo run
```

O servidor será iniciado em `http://localhost:8080`

## 📡 Documentação da API

### Base URL
```
http://localhost:8080
```

### Endpoints

#### Health Check
```http
GET /health
```
**Resposta:**
```json
{"status": "healthy"}
```

#### Criar Usuário
```http
POST /users
Content-Type: application/json

{
  "name": "João Silva",
  "email": "joao@email.com"
}
```

**Resposta (201 Created):**
```json
{
  "id": "user_1699123456789",
  "name": "João Silva", 
  "email": "joao@email.com",
  "created_at": "2024-11-04T15:30:45.123Z"
}
```

#### Listar Todos os Usuários
```http
GET /users
```

**Resposta (200 OK):**
```json
[
  {
    "id": "user_1699123456789",
    "name": "João Silva",
    "email": "joao@email.com", 
    "created_at": "2024-11-04T15:30:45.123Z"
  }
]
```

#### Buscar Usuário por ID
```http
GET /users/{id}
```

**Resposta (200 OK):**
```json
{
  "id": "user_1699123456789",
  "name": "João Silva",
  "email": "joao@email.com",
  "created_at": "2024-11-04T15:30:45.123Z"
}
```

**Resposta (404 Not Found):**
```json
{"error": "User not found"}
```

#### Atualizar Usuário
```http
PUT /users/{id}
Content-Type: application/json

{
  "name": "João Santos",
  "email": "joao.santos@email.com"
}
```

**Resposta (200 OK):**
```json
{
  "id": "user_1699123456789",
  "name": "João Santos",
  "email": "joao.santos@email.com",
  "created_at": "2024-11-04T15:30:45.123Z"
}
```

#### Deletar Usuário
```http
DELETE /users/{id}
```

**Resposta (200 OK):**
```json
{"message": "User deleted successfully"}
```

## 🧪 Testando a API

### Usando curl

```bash
# Criar usuário
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Maria Silva", "email": "maria@email.com"}'

# Listar usuários
curl http://localhost:8080/users

# Buscar usuário específico
curl http://localhost:8080/users/user_1699123456789

# Atualizar usuário
curl -X PUT http://localhost:8080/users/user_1699123456789 \
  -H "Content-Type: application/json" \
  -d '{"name": "Maria Santos"}'

# Deletar usuário
curl -X DELETE http://localhost:8080/users/user_1699123456789
```

### Usando HTTPie

```bash
# Criar usuário
http POST localhost:8080/users name="Asdrubal Silva" email="asdrubal@email.com"

# Listar usuários
http GET localhost:8080/users

# Atualizar usuário
http PUT localhost:8080/users/user_123 name="Noskralc Santos"
```

## 🚀 Deploy

### Railway.app

Este projeto está configurado para deploy automático no Railway.app:

1. Conecte seu repositório GitHub ao Railway
2. Configure a variável de ambiente `PORT` (Railway define automaticamente)
3. O deploy será feito automaticamente a cada push

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_api_crud /usr/local/bin/rust_api_crud
EXPOSE 8080
CMD ["rust_api_crud"]
```

## 🔍 Conceitos Avançados Demonstrados

### Ownership e Borrowing
```rust
// O sistema de ownership do Rust garante segurança de memória
let users = Arc::new(Mutex::new(HashMap::new()));
let users_clone = Arc::clone(&users); // Referência atômica, não cópia
```

### Async/Await
```rust
// Operações não-bloqueantes para alta concorrência
async fn create_user(mut req: Request<AppState>) -> Result {
    let users = req.state().users.lock().await; // Não bloqueia a thread
    // ...
}
```

### Pattern Matching
```rust
// Tratamento elegante de casos opcionais
match users.get(&user_id) {
    Some(user) => { /* usuário encontrado */ },
    None => { /* usuário não encontrado */ }
}
```

## 🤝 Contribuindo

1. Faça um fork do projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📈 Roadmap

- [ ] Autenticação JWT
- [ ] Integração com PostgreSQL
- [ ] Documentação OpenAPI/Swagger
- [ ] Rate limiting
- [ ] Logs estruturados
- [ ] Métricas e observabilidade
- [ ] Testes de integração
- [ ] CI/CD com GitHub Actions

## 📝 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

```
MIT License

Copyright (c) 2025 catitodev

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 👨‍💻 Desenvolvedor

**catitodev**
- GitHub: [@catitodev](https://github.com/catitodev)
- LinkedIn: [clarksonbartalini](https://linkedin.com/in/clarksonbartalini)

---

⭐ Se este projeto te ajudou, considere dar uma estrela no repositório!

## 📚 Recursos de Aprendizado

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tide Documentation](https://docs.rs/tide/)
- [Serde Documentation](https://serde.rs/)

---

*Construído com ❤️ e muito ☕ usando Rust 🦀*
