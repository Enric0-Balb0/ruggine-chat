# Ruggine Server - Docker Deployment Guide

This guide explains how to deploy the Ruggine chat server using Docker containers, following the requirements specified in the project documentation.

## Architecture Overview

The Ruggine application follows a containerized architecture with:

- **Rust Application Server**: Main chat server built with Axum framework
- **PostgreSQL 17 Database**: Persistent data storage in Docker container
- **Docker Compose**: Container orchestration and networking
- **Multi-stage Docker Build**: Optimized for minimal binary size

## Prerequisites

- Docker Engine 28.0+
- Docker Compose 2.34+
- 8GB RAM minimum
- 10GB available disk space

## Quick Start

### Production Deployment

1. **Clone and navigate to server directory**:
   ```bash
   cd ruggine_server
   ```

2. **Set up environment variables**:
   ```bash
   # Edit .env.docker with your production values
   ```

3. **Access the application**:
   - Server: http://localhost:8002
   - Database: postgresql://localhost:5432

## Manual Docker Commands

### Production

```bash
# Build and start production services
docker-compose -f docker-compose.prod.yml up -d --build
docker-compose -f docker-compose.prod.yml up -d

# Start with proxy
docker-compose -f docker-compose.prod.yml up -d ruggine_proxy

# 

# View logs
docker-compose logs -f

# Stop services
docker-compose -f docker-compose.prod.yml down

# Stop and remove volumes (WARNING: deletes data)
docker-compose -f docker-compose.prod.yml down -v
```

### Development

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View development logs
docker-compose -f docker-compose.dev.yml logs -f ruggine_dev_server

# Stop development environment
docker-compose -f docker-compose.dev.yml down
```

## Configuration

### Environment Variables

Key environment variables for production (set in `.env`):

```bash
# Database
POSTGRES_PASSWORD=your_secure_password

# JWT Security
JWT_SECRET=your_super_secure_jwt_secret

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8002

# Performance Monitoring
LOG_IN_MILLISECONDS=120000
```

### Performance Optimization

The Docker setup includes several optimizations:

- **Multi-stage build** for minimal image size (~30MB target)
- **Alpine Linux** base images for security and size
- **Non-root user** execution for security
- **Resource limits** to prevent resource exhaustion
- **Health checks** for service monitoring

## Networking

### Port Configuration

- **Production**:
  - Application: 8002
  - Database: 5432

- **Development**:
  - Application: 8003
  - Database: 5433
  - pgAdmin: 8080

### Internal Networking

Services communicate through Docker networks:
- Production: `ruggine_network`
- Development: `ruggine_dev_network`

## Data Persistence

### Volumes

- **Database Data**: Persistent PostgreSQL data
- **Development Caches**: Cargo and target caches for faster builds

### Backup

To backup the database:

```bash
# Production backup
docker exec ruggine_database pg_dump -U ruggine_user ruggine_prod > backup.sql

# Development backup
docker exec ruggine_dev_database pg_dump -U ruggine_dev_user ruggine_dev > dev_backup.sql
```

### Restore

```bash
# Restore production database
docker exec -i ruggine_database psql -U ruggine_user ruggine_prod < backup.sql
```

## Monitoring and Health Checks

### Service Health

All services include health checks:

```bash
# Check service status
docker-compose ps

# View detailed health status
docker inspect ruggine_app_server | grep -A 10 Health
```

### Application Logs

```bash
# Follow application logs
docker-compose logs -f ruggine_server

# View specific service logs
docker-compose logs ruggine_db
```

### Performance Monitoring

The application logs CPU usage every 2 minutes as per requirements:

```bash
# Monitor CPU usage logs
docker-compose logs -f ruggine_server | grep "CPU"
```

## Troubleshooting

### Common Issues

1. **Port conflicts**:
   ```bash
   # Check what's using the port
   netstat -tulpn | grep :8002
   
   # Or use different ports in docker-compose.yml
   ```

2. **Database connection issues**:
   ```bash
   # Check database health
   docker-compose exec ruggine_db pg_isready -U ruggine_user
   
   # View database logs
   docker-compose logs ruggine_db
   ```

3. **Build failures**:
   ```bash
   # Clean build with no cache
   docker-compose build --no-cache
   
   # Check Dockerfile and dependencies
   ```

4. **Memory issues**:
   ```bash
   # Check container resource usage
   docker stats
   
   # Adjust resource limits in docker-compose.yml
   ```

### Useful Commands

```bash
# View container resource usage
docker stats

# Execute commands in containers
docker-compose exec ruggine_server sh
docker-compose exec ruggine_db psql -U ruggine_user -d ruggine_prod

# Clean up unused Docker resources
docker system prune -f

# Reset everything (WARNING: deletes all data)
docker-compose down -v
docker system prune -a -f
```

## Security Considerations

1. **Change default passwords** in `.env` file
2. **Use strong JWT secrets** for production
3. **Firewall configuration** for production deployments
4. **Regular security updates** of base images
5. **Non-root user** execution in containers

## Development Features

Development environment includes:

- **Hot reload** with cargo-watch
- **pgAdmin** for database administration
- **Separate network** and ports
- **Volume caching** for faster rebuilds
- **Debug logging** enabled

This Docker setup ensures the Ruggine application meets the performance and deployment requirements specified in the project documentation.
