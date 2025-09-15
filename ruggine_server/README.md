# Ruggine Server

**Ruggine** is a modern, high-performance chat server built with Rust and Axum framework. It provides real-time messaging capabilities, user management, group chat functionality, and comprehensive performance monitoring.

## 🚀 Features

- **Real-time Chat**: WebSocket-based messaging system for instant communication
- **User Management**: Registration, authentication, and profile management
- **Group Chat System**: Create and manage group conversations with invitation system
- **Role-based Access**: Admin and member roles with different permissions
- **Performance Monitoring**: Built-in CPU usage logging and monitoring
- **API Documentation**: Automatic OpenAPI/Swagger documentation
- **Database Integration**: PostgreSQL with SQLx for type-safe database operations
- **JWT Authentication**: Secure token-based authentication system

## 📋 Prerequisites

- **Rust** 1.70+ (for development)
- **Docker** 28.0+ and Docker Compose 2.34+
- **PostgreSQL** 17+ (if running locally without Docker)
- **12GB RAM** minimum
- **100GB disk space** available

## 🛠 Installation & Setup

### Quick Start with Docker (Recommended)

1. **Clone the repository and navigate to server directory**:
   ```bash
   cd ruggine_server
   ```

2. **Set up environment variables**:
   ```bash
   # Copy example environment file
   cp .env.example .env
   # Copy example environment docker file if you want to run with docker
   cp .env.docker .env
   
   # Edit .env with your configuration
   # Required variables:
   # - POSTGRES_PASSWORD
   # - JWT_SECRET
   # - PORT (default: 8002)
   ```

3. **Start the application**:
   ```bash
   # Production deployment
   docker-compose up -d
   
   # Development with live reload
   docker-compose -f docker-compose.prod.yml up -d
   ```

4. **Access the application**:
   - **Server API**: http://localhost:8002
   - **API Documentation**: http://localhost:8002/swagger-ui
   - **Health Check**: http://localhost:8002/api/health

### Manual Installation (Development)

1. **Install Rust dependencies**:
   ```bash
   cargo build
   ```

2. **Set up PostgreSQL database**:
   ```bash
   # Run the query in the seeds folder to initaliaze a db
   ```

3. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit DATABASE_URL and other variables
   ```

4. **Run the server**:
   ```bash
   cargo run
   ```

## 🏗 Architecture

### Core Components

- **Axum Web Framework**: High-performance async web server
- **PostgreSQL Database**: Persistent data storage with ACID compliance
- **WebSocket System**: Real-time bidirectional communication
- **JWT Authentication**: Stateless authentication with role-based access
- **CPU Monitoring**: Background service for performance tracking

### Database Schema

The application uses the following main entities:

- **Users**: User accounts with authentication and profile information
- **Group Chats**: Chat rooms with metadata and ownership
- **Text Messages**: Chat messages with timestamps and delivery status
- **Group Memberships**: User participation in groups with roles
- **Invitations**: Group invitation system with status tracking
- **CPU Usage Logs**: Performance monitoring data

See `doc/database.md` for detailed ER diagram and schema information.

## 🔐 Authentication

The server uses JWT (JSON Web Tokens) for authentication:

1. **Login**: Send credentials to `/api/auth/login`
2. **Receive Token**: Get JWT token in response
3. **Use Token**: Include token in `Authorization: Bearer <token>` header
4. **Token Expiration**: Default expiration is 24 hours (configurable)

### Example Authentication Flow

```bash
# 1. Register a new user
curl -X POST http://localhost:8002/api/user/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "secure_password",
    "first_name": "John",
    "last_name": "Doe",
    "gender": "male"
  }'

# 2. Login to get token
curl -X POST http://localhost:8002/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "password": "secure_password"
  }'

# 3. Use token for authenticated requests
curl -X GET http://localhost:8002/api/user/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 💬 Real-time Messaging

### WebSocket Connection

Connect to the WebSocket endpoint for real-time messaging:

```javascript
const token = "YOUR_JWT_TOKEN";
const ws = new WebSocket(`ws://localhost:8002/api/ws/chat?token=${token}`);

ws.onopen = () => {
    console.log("Connected to chat");
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log("New message:", message);
};
```

### Message Types

The WebSocket system handles various message types:
- **Text Messages**: Regular chat messages
- **Group Notifications**: User join/leave events
- **Invitation Notifications**: New invitations received
- **System Messages**: Server announcements

## 🔧 Configuration

### Environment Variables

Key configuration options in `.env`:

```bash
# Database
DATABASE_URL=postgres://user:password@localhost:5432/ruggine
POSTGRES_PASSWORD=your_secure_password

# Server
PORT=8002
SERVER_HOST=0.0.0.0

# JWT Authentication
JWT_SECRET=your_super_secure_jwt_secret_change_this_in_production
JWT_TTL_IN_MINUTES=1440

# Performance Monitoring
LOG_IN_MILLISECONDS=120000  # CPU monitoring interval

# Application
RUST_ENV=production
RUST_LOG=info
```

### Performance Tuning

The server includes built-in performance monitoring:

- **CPU Usage Logging**: Configurable interval monitoring
- **Memory Management**: Efficient Rust memory handling
- **Connection Pooling**: Database connection optimization
- **WebSocket Management**: Efficient real-time connection handling

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run with coverage
cargo llvm-cov --html --output-dir coverage

# Run specific test categories
cargo test --lib                    # Unit tests
cargo test --test integration_*     # Integration tests
```

### Test Categories

- **Unit Tests**: Individual component testing
- **Integration Tests**: API endpoint testing
- **E2E Tests**: Full workflow testing
- **WebSocket Tests**: Real-time communication testing

## 🐳 Docker Deployment

### Production Deployment

```bash
# Build and deploy
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose logs -f ruggine_server

# Stop services
docker-compose down
```

### Development Environment

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# Watch logs
docker-compose -f docker-compose.dev.yml logs -f
```

### Container Architecture

- **Multi-stage Build**: Optimized for minimal image size
- **Alpine Linux**: Lightweight base image
- **Health Checks**: Automatic container health monitoring
- **Volume Persistence**: Database data persistence
- **Network Isolation**: Secure container networking

## 📊 Monitoring & Logging

### CPU Usage Monitoring

The server automatically monitors CPU usage:

- **Configurable Intervals**: Set monitoring frequency
- **Database Storage**: Historical performance data
- **API Access**: Retrieve logs via REST API
- **Admin Only**: Restricted access to performance data

### Application Logging

```bash
# View real-time logs
docker-compose logs -f ruggine_server

# Set log level
export RUST_LOG=debug
```

Log levels available: `error`, `warn`, `info`, `debug`, `trace`

## 🔒 Security Features

- **JWT Authentication**: Secure token-based auth
- **Password Hashing**: BCrypt password protection
- **CORS Configuration**: Cross-origin request management
- **Input Validation**: Request data validation
- **SQL Injection Protection**: Type-safe database queries
- **Rate Limiting**: (Configure as needed)

## 📚 API Documentation

Interactive API documentation is available at:
- **Swagger UI**: http://localhost:8002/swagger-ui
- **OpenAPI JSON**: http://localhost:8002/api-docs/openapi.json

The documentation includes:
- Complete endpoint reference
- Request/response schemas
- Authentication requirements
- Example requests and responses

## 📄 License

This project is part of the university coursework for "Programmazione di sistema" and is developed for educational purposes.

## 📞 Support

For technical support or questions:
- Check the troubleshooting section
- Review API documentation
- Examine log files for error details
- Verify configuration settings

---

**Ruggine Server** - Built with ❤️ using Rust, Axum, and modern web technologies.
