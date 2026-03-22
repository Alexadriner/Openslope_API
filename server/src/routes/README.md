# OpenSlope API Routes

This directory contains all the HTTP route handlers for the OpenSlope API. The routes are organized by domain and provide comprehensive RESTful endpoints for managing ski resorts, lifts, slopes, status information, and user authentication.

## Directory Structure

```
routes/
├── mod.rs              # Module registry and documentation
├── resorts.rs          # Ski resort management endpoints
├── lifts.rs            # Lift information and management
├── slopes.rs           # Slope information and management
├── status.rs           # Scraping status and operational data
├── auth.rs             # User authentication and API key management
└── README.md           # This documentation file
```

## Route Categories

### 1. Authentication (`auth.rs`)

User authentication and API key management with bcrypt security.

#### Public Endpoints (No Authentication Required):
- **POST /signup** - Register new user account
- **POST /signin** - Authenticate user and receive API key

#### Protected Endpoints (API Key Required):
- **GET /me** - Get current user information using API key

#### Security Features:
- **Password Hashing**: bcrypt with automatic salt generation
- **API Key Generation**: Cryptographically secure random keys
- **Input Validation**: Comprehensive validation of user input
- **Flexible Authentication**: Support for email or username login

#### Authentication Flow:
1. User registration with email, username, and password
2. Password hashing and API key generation
3. Login with email/username and password
4. New API key generation on successful login
5. API key verification for protected endpoints

### 2. Resorts (`resorts.rs`)

Complete ski resort management with hierarchical data including lifts and slopes.

#### Protected Endpoints (API Key Required):
- **GET /resorts** - List all resorts (supports `?summary=1` for basic info)
- **GET /resorts/{id}** - Get detailed resort information with all related data
- **POST /resorts** - Create a new resort
- **PUT /resorts/{id}** - Update an existing resort
- **DELETE /resorts/{id}** - Delete a resort

#### Key Features:
- **Hierarchical Data**: Resorts include nested lifts and slopes
- **Summary Mode**: Optional lightweight response with just ID and name
- **Batch Loading**: Efficient loading of related data using HashMap indexing
- **Complex Geometry**: Support for lift/slope path geometry with GeoJSON parsing

#### Example Response:
```json
{
  "id": "resort_123",
  "name": "Example Ski Resort",
  "geography": {
    "continent": "Europe",
    "country": "Austria",
    "region": "Tyrol",
    "coordinates": {
      "latitude": 47.2628,
      "longitude": 11.3936
    }
  },
  "altitude": {
    "village_m": 1200,
    "min_m": 1100,
    "max_m": 2500
  },
  "lifts": [...],
  "slopes": [...]
}
```

### 3. Slopes (`slopes.rs`)

Individual slope management with difficulty classification and complex geometry.

#### Protected Endpoints (API Key Required):
- **GET /slopes** - List all slopes with detailed information
- **GET /slopes/{id}** - Get specific slope details
- **POST /slopes** - Create a new slope
- **PUT /slopes/{id}** - Update an existing slope
- **DELETE /slopes/{id}** - Delete a slope
- **GET /slopes/by_resort/{resort_id}** - Get all slopes for a specific resort
- **DELETE /slopes/by_resort/{resort_id}** - Delete all slopes for a resort

#### Key Features:
- **Difficulty Classification**: Green, Blue, Red, Black levels
- **Complex Geometry**: Start/end points plus optional path geometry
- **Technical Specifications**: Length, gradients, vertical drop
- **Skiing Features**: Snowmaking, night skiing, family-friendly, race slope indicators

#### Difficulty Levels:
- **Green**: Beginner slopes (< 25% gradient)
- **Blue**: Intermediate slopes (25-40% gradient)
- **Red**: Advanced slopes (40-60% gradient)
- **Black**: Expert slopes (> 60% gradient)

### 4. Lifts (`lifts.rs`)

Individual lift management with technical specifications and operational status.

#### Protected Endpoints (API Key Required):
- **GET /lifts** - List all lifts with detailed information
- **GET /lifts/{id}** - Get specific lift details
- **POST /lifts** - Create a new lift
- **PUT /lifts/{id}** - Update an existing lift
- **DELETE /lifts/{id}** - Delete a lift
- **GET /lifts/by_resort/{resort_id}** - Get all lifts for a specific resort
- **DELETE /lifts/by_resort/{resort_id}** - Delete all lifts for a resort

#### Key Features:
- **Technical Specifications**: Capacity, seats, year built, altitude data
- **Geographical Data**: Precise start/end coordinates for mapping
- **Status Management**: Operational status, planned times, notes
- **Source Integration**: Support for multiple data sources (OSM, official)

#### Lift Types:
- Chairlift (detachable/fixed-grip)
- Gondola (enclosed cabins)
- T-bar (surface lifts)
- Surface lift (magic carpet)
- Funitel (cable transport)

### 5. Status (`status.rs`)

Scraping status and real-time operational data for resorts.

#### Protected Endpoints (API Key Required):
- **GET /scrape-runs** - List all scraping operations (supports `?resort_id` and `?limit`)
- **GET /scrape-runs/{id}** - Get specific scraping run details
- **GET /status-snapshots** - List all status snapshots (supports `?resort_id` and `?limit`)
- **GET /resorts/{resort_id}/status-snapshots** - Get snapshots for specific resort

#### Key Features:
- **Scraping History**: Complete history of all scraping operations
- **Status Snapshots**: Real-time operational data (lifts, slopes, snow, temperature)
- **Pagination Control**: Configurable limits (1-500 records, default 100)
- **Resort Filtering**: All endpoints support optional resort-specific filtering

#### Status Data:
- **Lift Metrics**: Open/total lift counts
- **Slope Metrics**: Open/total slope counts
- **Snow Data**: Valley/mountain depth, new snow measurements
- **Temperature Data**: Valley/mountain temperature readings

## Data Flow Architecture

```
HTTP Request
     ↓
Route Handler (routes/*.rs)
     ↓
Database Operations (sqlx queries)
     ↓
Data Transformation (model mapping)
     ↓
JSON Response (serde serialization)
```

## Error Handling Strategy

All route handlers implement consistent error handling:

- **400 Bad Request**: Invalid input data or malformed requests
- **401 Unauthorized**: Missing or invalid authentication
- **404 Not Found**: Resource not found for specific IDs
- **500 Internal Server Error**: Database errors or unexpected failures

## Security Considerations

### Authentication
- All routes (except auth endpoints) require valid API keys
- API keys are passed via query parameters (consider headers for future versions)
- Passwords are never stored in plaintext
- API keys are hashed before database storage

### Input Validation
- All input data is validated before processing
- SQL injection prevention using prepared statements with SQLx
- Generic error messages to prevent information leakage
- Consistent timing for authentication operations

### Data Protection
- Sensitive data is not exposed in error responses
- API keys are rotated on each successful login
- Database connections use proper credential management

## Performance Optimizations

### Database Efficiency
- **Connection Pooling**: MySQL connection pools for efficiency
- **Query Optimization**: Optimized SQL with proper column selection
- **Index Usage**: Queries designed to use database indexes effectively
- **Batch Operations**: Efficient loading of related data

### Response Optimization
- **Pagination**: Limits on list endpoints to prevent large data transfers
- **Selective Loading**: Only load related data when needed
- **Efficient Serialization**: Optimized JSON serialization with serde

## Usage Examples

### Basic Operations

```bash
# Get all resorts
curl GET /resorts

# Get resort summary only
curl GET /resorts?summary=1

# Get specific resort with all data
curl GET /resorts/resort_123

# Get all lifts for a resort
curl GET /lifts/by_resort/resort_123

# Get all slopes for a resort
curl GET /slopes/by_resort/resort_123
```

### Authentication

```bash
# User registration
curl -X POST /signup \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "username": "ski_enthusiast", "password": "secure_password123"}'

# User login
curl -X POST /signin \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure_password123"}'

# Get user info (requires API key)
curl GET "/me?api_key=your_api_key_here"
```

### Status Monitoring

```bash
# Get recent scraping runs
curl GET "/scrape-runs?limit=10"

# Get scraping runs for specific resort
curl GET "/scrape-runs?resort_id=resort_123"

# Get recent status snapshots
curl GET "/status-snapshots?limit=5"

# Get snapshots for specific resort
curl GET "/resorts/resort_123/status-snapshots"
```

## Testing

Routes should be tested for:
- **HTTP Status Codes**: Proper status codes for all scenarios
- **Data Validation**: Input validation and error handling
- **Authentication**: API key verification and user permissions
- **Performance**: Response times and database query efficiency
- **Edge Cases**: Empty results, malformed data, missing resources

## Future Enhancements

### Authentication
- API key expiration and refresh mechanisms
- Two-factor authentication support
- Account lockout after failed attempts
- Audit logging for security monitoring

### Performance
- Response caching for frequently accessed data
- Database query optimization and indexing
- Rate limiting for API endpoints
- CDN integration for static assets

### Features
- Real-time updates via WebSocket endpoints
- Analytics and usage statistics
- Admin dashboard for system management
- Webhook endpoints for external integrations

## Dependencies

- **actix-web**: HTTP server framework
- **serde**: Serialization/deserialization
- **sqlx**: Async SQL toolkit
- **bcrypt**: Password hashing
- **url**: URL parsing for authentication

## License

This documentation and code is part of the OpenSlope project.

---

*Last updated: March 2026*
*Version: 1.0.0*