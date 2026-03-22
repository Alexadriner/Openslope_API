# OpenSlope API Server Source Code

This directory contains the complete source code for the OpenSlope API server, a comprehensive RESTful API for managing ski resort data including resorts, slopes, lifts, and user authentication.

## Overview

The OpenSlope API server is built using Rust with the Actix Web framework, providing a high-performance, secure, and scalable backend for ski resort management and data access.

### Key Features

- **RESTful API Design**: Clean, intuitive API endpoints following REST principles
- **Authentication & Authorization**: API key-based authentication with rate limiting
- **Database Integration**: MySQL database with comprehensive data models
- **Security**: Industry-standard security practices including password hashing and rate limiting
- **Modular Architecture**: Clean separation of concerns with dedicated modules
- **Comprehensive Documentation**: Extensive inline documentation and API documentation

## Directory Structure

```
server/src/
├── main.rs              # Application entry point and server configuration
├── auth.rs              # Authentication middleware and authorization logic
├── user_service.rs      # User management service functions
├── db/                  # Database layer and queries
│   ├── mod.rs
│   └── resort_queries.rs
├── models/              # Data models and database schemas
│   ├── mod.rs
│   ├── db.rs
│   ├── resort.rs
│   ├── slope.rs
│   └── lift.rs
├── routes/              # HTTP endpoint handlers
│   ├── mod.rs
│   ├── auth.rs
│   ├── resorts.rs
│   ├── slopes.rs
│   ├── lifts.rs
│   └── status.rs
├── security/            # Security utilities and authentication
│   ├── mod.rs
│   ├── api_key.rs
│   ├── hash.rs
│   ├── subscription.rs
│   └── jwt.rs
└── services/            # Business logic services
    ├── mod.rs
    └── user_service.rs
```

## Core Components

### 1. Main Application (`main.rs`)

The main entry point that:
- Loads environment variables and configuration
- Establishes database connections
- Configures CORS and middleware
- Sets up all API routes and endpoints
- Starts the HTTP server on port 8080

**Key Features:**
- Environment-based configuration
- Database connection pooling
- CORS middleware for cross-origin requests
- Structured logging and error handling
- Comprehensive API endpoint documentation

### 2. Authentication System (`auth.rs`)

Comprehensive authentication middleware providing:
- **Multi-Source API Key Support**: Bearer tokens and query parameters
- **Rate Limiting**: Subscription-based rate limits with time windows
- **Administrative Privileges**: Role-based access control
- **Security Features**: Constant-time verification and secure storage

**Authentication Methods:**
```bash
# Bearer Token (Recommended)
curl -H "Authorization: Bearer your_api_key" http://localhost:8080/resorts

# Query Parameter (GET requests only)
curl "http://localhost:8080/resorts?api_key=your_api_key"
```

### 3. Data Models (`models/`)

Type-safe data models representing the domain entities:

#### User Model
- User registration and authentication
- API key management
- Subscription-based access control
- Administrative privileges

#### Resort Model
- Comprehensive resort information
- Geographic coordinates and metadata
- Status tracking and maintenance

#### Slope Model
- Slope details and characteristics
- Difficulty levels and measurements
- Status and maintenance information

#### Lift Model
- Lift specifications and status
- Operational information
- Maintenance tracking

### 4. Database Layer (`db/`)

Database abstraction layer with:
- **Connection Pooling**: Efficient database connection management
- **Query Optimization**: Prepared statements and optimized queries
- **Error Handling**: Comprehensive database error handling
- **Transaction Support**: Atomic database operations

### 5. HTTP Routes (`routes/`)

RESTful API endpoints organized by entity type:

#### Public Endpoints (No Authentication Required)
- **POST /signup** - User registration
- **POST /signin** - User login

#### Protected Endpoints (API Key Required)
- **GET /me** - Get current user info
- **GET /resorts** - Get all resorts
- **GET /resorts/{id}** - Get specific resort
- **POST /resorts** - Create new resort
- **PUT /resorts/{id}** - Update resort
- **DELETE /resorts/{id}** - Delete resort
- **GET /slopes** - Get all slopes
- **GET /slopes/{id}** - Get specific slope
- **POST /slopes** - Create new slope
- **PUT /slopes/{id}** - Update slope
- **DELETE /slopes/{id}** - Delete slope
- **GET /lifts** - Get all lifts
- **GET /lifts/{id}** - Get specific lift
- **POST /lifts** - Create new lift
- **PUT /lifts/{id}** - Update lift
- **DELETE /lifts/{id}** - Delete lift
- **GET /scrape-runs** - List scraping operations
- **GET /status-snapshots** - Get status snapshots

### 6. Security Module (`security/`)

Comprehensive security utilities:
- **API Key Generation**: Cryptographically secure key generation
- **Password Hashing**: Argon2-based password security
- **JWT Tokens**: JSON Web Token creation and management
- **Rate Limiting**: Subscription-based usage control

### 7. Services Layer (`services/`)

Business logic services providing:
- **User Management**: Registration, authentication, and profile management
- **Data Processing**: Complex business operations and data transformations
- **Integration Logic**: Coordination between different system components

## API Usage Examples

### User Registration
```bash
curl -X POST http://localhost:8080/signup \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "secure_password123"
  }'
```

### Get All Resorts
```bash
curl "http://localhost:8080/resorts?api_key=your_api_key_here"
```

### Create New Resort (Admin Only)
```bash
curl -X POST http://localhost:8080/resorts \
  -H "Authorization: Bearer admin_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Ski Resort",
    "location": "Mountain Range",
    "latitude": 47.12345,
    "longitude": 11.67890,
    "website": "https://newresort.com"
  }'
```

## Security Features

### Authentication & Authorization
- **API Key Authentication**: Secure API key generation and verification
- **Role-Based Access**: Administrative privileges for data modification
- **Rate Limiting**: Subscription-based request limits
- **Input Validation**: Comprehensive input sanitization and validation

### Data Security
- **Password Hashing**: Argon2 algorithm with automatic salt generation
- **API Key Security**: Cryptographically secure key generation
- **Database Security**: Prepared statements and parameterized queries
- **Error Handling**: Generic error messages to prevent information leakage

### Rate Limiting
- **Subscription-Based**: Different limits based on user plans
- **Time Windows**: Separate minute and monthly rate limiting
- **Automatic Reset**: Time-based limit resets
- **Real-time Tracking**: Live request counting and enforcement

## Database Schema

The application uses a MySQL database with the following key tables:

### Users Table
```sql
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE,
    subscription VARCHAR(50) DEFAULT 'Free',
    requests_minute INT DEFAULT 0,
    requests_month INT DEFAULT 0,
    last_request_minute TIMESTAMP,
    last_request_month DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Resorts Table
```sql
CREATE TABLE resorts (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    latitude DECIMAL(10,8),
    longitude DECIMAL(11,8),
    website VARCHAR(255),
    status VARCHAR(50) DEFAULT 'unknown',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Development Setup

### Prerequisites
- Rust 1.70+
- MySQL database server
- Cargo package manager

### Installation
1. Clone the repository
2. Install dependencies: `cargo build`
3. Set up environment variables (see `.env.example`)
4. Run database migrations
5. Start the server: `cargo run`

### Environment Configuration
Create a `.env` file with the following variables:
```
DATABASE_URL=mysql://username:password@localhost:3306/openslope_db
JWT_SECRET=your_jwt_secret_key_here
```

## Performance Considerations

### Database Optimization
- **Connection Pooling**: Efficient database connection management
- **Index Usage**: Proper indexing for frequently queried fields
- **Query Optimization**: Prepared statements and optimized queries
- **Async Operations**: Non-blocking database operations

### Memory Management
- **Resource Cleanup**: Proper cleanup of database connections
- **Memory Limits**: Appropriate memory limits for large operations
- **Efficient Data Structures**: Optimized data structures for performance

### Caching Strategies
- **Frequently Accessed Data**: Consider caching for frequently accessed data
- **Cache Invalidation**: Proper cache invalidation strategies
- **TTL Management**: Appropriate time-to-live values

## Testing

The application includes comprehensive testing:
- **Unit Tests**: Individual function testing
- **Integration Tests**: End-to-end testing with real database
- **Mock Testing**: External dependency mocking
- **Error Testing**: Error condition and edge case testing

Run tests with: `cargo test`

## Monitoring and Observability

### Metrics to Monitor
- API response times and throughput
- Database connection pool usage
- Authentication success/failure rates
- Rate limit violations
- Error rates and types

### Logging
- Structured logging for better debugging
- Security event logging
- Performance metric logging
- Error and exception logging

### Health Checks
- Database connectivity
- API endpoint availability
- Authentication system status
- Rate limiting functionality

## Security Best Practices

### Development
- Use HTTPS in production
- Never commit secrets to version control
- Use environment variables for configuration
- Implement proper input validation

### Production
- Regular security updates and patches
- Database access control and monitoring
- API key rotation policies
- Security event monitoring and alerting

### Compliance
- GDPR compliance for user data
- OWASP security guidelines
- Industry-standard authentication practices
- Secure API design principles

## Future Enhancements

### Planned Features
- **OAuth Integration**: Support for OAuth 2.0 authentication
- **Two-Factor Authentication**: Enhanced security with 2FA
- **Real-time Updates**: WebSocket support for live updates
- **Advanced Analytics**: Comprehensive usage analytics
- **Mobile SDKs**: Client libraries for mobile applications
- **GraphQL Support**: Alternative API interface
- **Microservices Architecture**: Service decomposition for scalability

### Performance Improvements
- **Caching Layer**: Redis-based caching for improved performance
- **Load Balancing**: Multi-instance deployment support
- **Database Sharding**: Horizontal scaling for large datasets
- **CDN Integration**: Content delivery network for static assets

## Contributing

### Code Style
- Follow Rust coding standards
- Use meaningful variable and function names
- Include comprehensive documentation
- Write tests for new features

### Pull Request Process
1. Create a feature branch from `main`
2. Make changes with comprehensive tests
3. Ensure all tests pass
4. Submit pull request with detailed description
5. Address review feedback

### Security Considerations
- Never commit secrets or credentials
- Use secure coding practices
- Validate all user inputs
- Follow security best practices

## Support and Documentation

### API Documentation
- Comprehensive inline documentation
- API endpoint documentation in `main.rs`
- Example usage in route files
- Security documentation in `security/` module

### Troubleshooting
- Check server logs for error details
- Verify database connectivity
- Ensure proper API key usage
- Monitor rate limit status

### Contact
For support and questions:
- Review the inline documentation
- Check the API examples
- Consult the security and testing documentation
- Report issues through appropriate channels

---

**Author**: OpenSlope Team  
**Version**: 1.0.0  
**Last Updated**: March 2026