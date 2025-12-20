# Kanban Board - Backend API

A high-performance REST API backend for a collaborative Kanban board application built with Rust. Built following Clean Architecture, Domain-Driven Design, and Event-Driven Architecture principles, it offers robust user authentication, board management, and WebSocket-based real-time updates.

## :sparkles: Features

<details>
  <summary>Implemented</summary>

  #### Authentication & Authorization
  - [x] User registration with email verification
  - [x] Login/logout with session management (Redis-backed)
  - [x] Password reset flow via email
  - [x] Account activation via email tokens
  - [x] Session-based authentication with cookies
  - [x] Protected endpoints with middleware

  #### User Management
  - [x] Get user profile information

  #### Board Management
  - [x] Create, read, update, delete boards
  - [x] Board ownership and member management
  - [x] Role-based permissions (Owner, Moderator, Member)
  - [x] Add/remove board members
  - [x] Update member roles
  - [x] List user's boards

  #### Column Management
  - [x] Create, read, update, delete columns
  - [x] Reorder columns using fractional indexing
  - [x] List columns by board
  - [x] Column positioning system

  #### Task Management
  - [x] Create, read, update, delete tasks
  - [x] Reorder tasks using fractional indexing
  - [x] List tasks by column
  - [x] Task positioning system
</details>

<details>
  <summary>Planned Features & Improvements</summary>

  #### To Be Done
  - [ ] Task assignments to specific users
  - [ ] Task due dates and priorities
  - [ ] Comments on tasks
  - [ ] File attachments to tasks
  - [ ] Board, column and task archiving
  - [ ] User profile update
  - [ ] User avatar management
  - [ ] Notification system
  - [ ] Search and filtering
  - [ ] CORS controls
  - [ ] Rate limiting
  - [ ] Unit and integration tests
</details>

## :building_construction: Architecture

This project follows Clean Architecture, Domain-Driven Design, and Event-Driven Architecture principles, organized into distinct layers with clear separation of concerns.

<details>
  <summary>Architecture Layers</summary>

  ```mermaid
  graph TB
    PL --> AL
    subgraph PL["Presentation Layer"]
        direction TB
        Controllers["HTTP Controllers"]
        Middleware["Middleware"]
        WebSocket["WebSocket Handler"]
        OpenAPI["OpenAPI Docs"]
    end

    AL --> DL
    subgraph AL["Application Layer"]
        direction TB
        AppServices["Application Services"]
        DTOs["DTOs"]
    end

    DL --> IL
    subgraph DL["Domain Layer"]
        direction TB
        Repositories["Repository Traits"]
        DomainServices["Domain Services"]
        Events["Events"]
        Entities["Domain Models"]
    end

    IL --> EXSD
    subgraph IL["Infrastructure Layer"]
        direction TB
        Persistence["Persistence<br/>SeaORM Repository Implementations"]
        CacheService["Cache Service<br/>(Redis)"]
        EmailService["Email Service<br/>(SMTP)"]
        EventBusImpl["Event Bus Implementation<br/>(In-Memory Event Bus)"]
    end

    subgraph EXSD["External Systems & Databases"]
        direction TB
        Database[("PostgreSQL<br/>Database")]
        Cache[("Redis<br/>Database")]
        SMTP["SMTP Server"]
    end

    classDef noWrap white-space:nowrap;
    class EXSD noWrap;
  ```
</details>

<details>
  <summary>Example Data Flows</summary>

  ```mermaid
  sequenceDiagram
    participant Client
    participant Controller
    participant Middleware
    participant AppService as Application Service
    participant DomainService as Domain Service
    participant Repository
    participant EventBus
    participant Database as PostgreSQL
    participant Redis
    participant Email as Email Service
    participant WebSocketService as WebSocket Service

    Note over Client,Email: User registration flow with email
    Client->>Controller: POST /api/auth/register
    Controller->>Middleware: RequireAuth (skip for auth)
    Middleware->>Controller: Continue
    Controller->>AppService: AuthService.register(dto)
    AppService->>Repository: UserRepository.exists_by_email()
    Repository->>Database: SELECT email FROM user
    Database-->>Repository: No existing user
    AppService->>AppService: Hash password (Argon2)
    AppService->>Repository: UserRepository.create(user)
    Repository->>Database: INSERT INTO user
    Database-->>Repository: User created
    AppService->>DomainService: TokenService.store_activation_token()
    DomainService->>Redis: SET activation_token with TTL
    Redis-->>DomainService: Token stored
    AppService->>DomainService: EmailService.send_activation_email()
    DomainService->>Email: Send SMTP email
    Email-->>DomainService: Email sent
    AppService-->>Controller: UserDto
    Controller-->>Client: 201 Created

    Note over Client,WebSocketService: Board update flow with events
    Client->>Controller: PUT /api/board/:boardId
    Controller->>Middleware: RequireAuth
    Middleware->>Redis: Validate session
    Redis-->>Middleware: Session valid
    Middleware->>Controller: User authenticated
    Controller->>AppService: BoardService.update(id, dto)
    AppService->>Repository: BoardMemberRepository.check_permission()
    Repository->>Database: SELECT role FROM board_member
    Database-->>Repository: User has permission
    AppService->>Repository: BoardRepository.update(board)
    Repository->>Database: UPDATE board SET ...
    Database-->>Repository: Board updated
    AppService->>EventBus: Publish BoardUpdatedEvent
    EventBus->>WebSocketService: Notify subscribers
    WebSocketService-->>Client: WebSocket: board.updated
    AppService-->>Controller: BoardDto
    Controller-->>Client: 200 OK
  ```
</details>

<details>
  <summary>Database Schema</summary>
  
  ```mermaid
  erDiagram
    USER ||--o{ BOARD : "owns"
    USER ||--o{ BOARD_MEMBER : "participates in"
    USER {
        uuid id PK "DEFAULT uuidv7()"
        varchar(254) email UK
        varchar password
        varchar(50) first_name
        varchar(50) last_name
        boolean is_active "DEFAULT false"
        timestamptz created_at "DEFAULT NOW()"
        timestamptz updated_at "DEFAULT NOW()"
    }

    BOARD ||--o{ BOARD_MEMBER : "has members"
    BOARD ||--o{ COLUMN : "contains"
    BOARD {
        uuid id PK "DEFAULT uuidv7()"
        varchar(100) name
        text description "Nullable"
        uuid owner_id FK "References USER.id (CASCADE)"
        timestamptz created_at "DEFAULT NOW()"
        timestamptz updated_at "DEFAULT NOW()"
    }

    BOARD_MEMBER {
        uuid id PK "DEFAULT uuidv7()"
        uuid board_id FK "References BOARD.id (CASCADE)"
        uuid user_id FK "References USER.id (CASCADE)"
        enum role "DEFAULT member (owner | moderator | member)"
        timestamptz created_at "DEFAULT NOW()"
        timestamptz updated_at "DEFAULT NOW()"
    }

    COLUMN ||--o{ TASK : "contains"
    COLUMN {
        uuid id PK "DEFAULT uuidv7()"
        varchar(100) name
        varchar(50) position "Fractional index"
        uuid board_id FK "References BOARD.id (CASCADE)"
        timestamptz created_at "DEFAULT NOW()"
        timestamptz updated_at "DEFAULT NOW()"
    }

    TASK {
        uuid id PK "DEFAULT uuidv7()"
        varchar(254) title
        text description "Nullable"
        varchar(50)[] tags "Nullable"
        varchar(50) position "Fractional index"
        uuid column_id FK "References COLUMN.id (CASCADE)"
        timestamptz created_at "DEFAULT NOW()"
        timestamptz updated_at "DEFAULT NOW()"
    }
  ```
</details>

<details>
  <summary>Project Structure</summary>

  ```
  kanban_be/
  ├── entity/                      # Database models (SeaORM entities)
  │   └── src/
  │       ├── board.rs
  │       ├── ...
  │       └── user.rs
  │
  ├── migration/                   # Database migrations
  │   └── src/
  │       ├── m20251102_200527_create_user_table.rs
  │       ├── ...
  │       └── m20251108_111856_create_board_member_table.rs
  │
  ├── src/
  │   ├── application/             # Application layer
  │   │   ├── dto/                 # Data Transfer Objects
  |   |   |   ├── auth_dto.rs
  │   │   |   ├── ...
  |   |   |   └── user_dto.rs
  │   │   └── services/            # Application services
  │   │       ├── auth_service.rs
  │   │       ├── ...
  │   │       └── websocket_service.rs
  │   │
  │   ├── domain/                  # Domain layer (business logic)
  │   │   ├── events/              # Domain events
  │   │   │   ├── board_event.rs
  │   │   │   └── event_bus.rs
  │   │   ├── repositories/        # Repository traits
  │   │   │   ├── board_member_repository.rs
  │   │   |   ├── ...
  │   │   │   └── user_repository.rs
  │   │   └── services/            # Domain services (traits)
  │   │       ├── email_service.rs
  │   │       └── token_service.rs
  │   │
  │   ├── infrastructure/          # Infrastructure layer
  │   │   ├── cache/               # Redis caching implementation
  │   │   │   └── token_service_impl.rs
  │   │   ├── email/               # Email service implementation
  │   │   │   └── email_service_impl.rs
  │   │   ├── event_bus/           # Event bus implementation
  │   │   │   └── in_memory_event_bus.rs
  │   │   └── persistence/         # Database repositories
  │   │       ├── board_member_repository_impl.rs
  │   │       ├── ...
  │   │       ├── user_repository_impl.rs
  │   │       └── database.rs      # Database connection
  │   │
  │   ├── presentation/            # Presentation layer
  │   │   ├── http/                # HTTP controllers
  │   │   │   ├── auth_controller.rs
  │   │   │   ├── ...
  │   │   │   ├── websocket_controller.rs
  │   │   │   ├── openapi.rs       # OpenAPI documentation
  │   │   │   └── server.rs        # Server configuration
  │   │   └── middleware/          # Custom middleware
  │   │       └── auth_middleware.rs
  │   │
  │   ├── shared/                  # Shared utilities
  │   │   ├── config/              # Application configuration
  │   │   │   ├── app_state.rs     # Global application state
  │   │   │   └── startup.rs       # Startup logic
  │   │   ├── utils/               # Utility functions
  │   │   │   ├── argon.rs
  │   │   │   ├── ...
  │   │   │   └── fractional_indexing.rs
  │   │   ├── error.rs             # Error types
  │   │   └── response.rs          # Response types
  │   │
  │   └── main.rs                  # Application entry point
  │
  ├── templates/                   # Email templates
  │   └── emails/
  │
  ├── Cargo.toml                   # Rust dependencies
  ├── LICENSE                      # Project license
  └── README.md                    # This file
  ```
</details>

## :rocket: Getting Started

### Prerequisities

- **Rust** 1.90+ (edition 2024)
- **PostgreSQL** 18+
- **Redis** 8+
- **SMTP Server** (Gmail, SendGrid, Mailgun or another)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/mysci4k/kanban_be.git
   cd kanban_be
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   ```
   
   <details>
     <summary>Variable list</summary>

     | Variable | Description | Required | Default | Example |
     |----------|-------------|----------|---------|---------|
     | `SERVER_ADDRESS` | Server bind address | No | 127.0.0.1 | 127.0.0.1 |
     | `SERVER_PORT` | Server port | No | 8080 | 8080 |
     | `DATABASE_URL` | PostgreSQL connection string | Yes | - | postgres://username:password@localhost:5432/kanban |
     | `REDIS_URL` | Redis connection string | Yes | - | redis://localhost:6379 |
     | `SESSION_KEY` | Secret key for session encryption | Yes | - | session-key-min-64-bytes-long |
     | `ACTIVE_TOKEN_TTL` | Active token TTL in seconds | No | 3600 | 3600 |
     | `PASSWORD_RESET_TOKEN_TTL` | Password reset token TTL in seconds | No | 3600 | 3600 |
     | `SMTP_SERVER` | SMTP server hostname | Yes | - | smtp.example.com |
     | `SMTP_USERNAME` | SMTP username | Yes | - | smtp-username |
     | `SMTP_PASSWORD` | SMTP password | Yes | - | smtp-password |
     | `FROM_EMAIL` | From email address | Yes | - | noreply@example.com |
     | `BASE_URL` | Frontend application URL | Yes | - | http://localhost:300 |
  </details>

3. **Set up PostgreSQL database and Redis server**

   <details>
     <summary>Option A: Local setup with Docker Compose</summary>

     ```bash
     # Start PostgreSQL and Redis containers
     docker-compose up -d

     # Verify services are running
     docker-compose ps

     # Stop services when done
     docker-compose down
     ```
   </details>

   <details>
     <summary>Option B: External hosted services</summary>

     | PostgreSQL | Redis |
     | ---------- | ----- |
     | [Supabase](https://supabase.com/) | [Upstash](https://upstash.com/) |
     | [Neon](https://neon.com/) | [Redis Cloud](https://redis.io/cloud/) |
     | [Railway](https://railway.com/) | [Railway](https://railway.com/) |
   </details>

4. **Run PostgreSQL database migrations**
   ```bash
   cargo run --package migration up
   ```

5. **Build and run the application**

   Development mode
   ```bash
   cargo run
   ```

   Development mode with auto-reload
   ```bash
   # Install bacon (one-time)
   cargo install --locked bacon
   # Run with auto-reload on file changes
   bacon run-long
   ```

   Production mode
   ```bash
   # Build the release binary
   cargo build --release
   # Run the optimized binary
   ./target/release/kanban_be

   # Alternatively, build and run in one command
   cargo run --release
   ```

The server will be available at `http://localhost:8080` (or your configured `SERVER_ADDRESS:SERVER_PORT`)

The interactive API documentation is available via Scalar UI at `http://localhost:8080/scalar`

## :handshake: Contributing

Contributions are welcome! Please see our [Contributing Guide](.github/contributing.md) for details on how to get started.

For questions or support, open an [issue](https://github.com/mysci4k/kanban_be/issues) or [discussion](https://github.com/mysci4k/kanban_be/discussions).

## :pencil: License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
