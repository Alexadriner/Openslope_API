# OpenSlope API Models

This directory contains all the data models used throughout the OpenSlope API system. The models are organized into two main categories: database models for direct SQL mapping and API response models for JSON serialization.

## Directory Structure

```
models/
├── mod.rs              # Module registry and documentation
├── db.rs              # Database row structures
├── resort.rs          # API response models for resorts
├── lift.rs            # API response models for lifts (empty - placeholder)
├── slope.rs           # API response models for slopes (empty - placeholder)
└── README.md          # This documentation file
```

## Model Categories

### 1. Database Models (`db.rs`)

Database models are designed for direct mapping from MySQL database tables using SQLx's `FromRow` derive macro. These models provide type-safe database interactions.

#### Key Features:
- **Direct SQL Mapping**: Automatic mapping from database rows
- **Type Safety**: Strong typing with proper null handling
- **Database Compatibility**: Field names match database column names exactly

#### Models:
- **`ResortRow`**: Complete resort information including geographical and operational data
- **`LiftRow`**: Individual lift information associated with resorts
- **`SlopeRow`**: Individual slope information associated with resorts

### 2. API Response Models (`resort.rs`)

API response models are designed for JSON serialization and provide comprehensive information about ski resorts and their facilities.

#### Key Features:
- **JSON Serialization**: Optimized for API responses
- **Comprehensive Data**: Hierarchical structure with nested information blocks
- **Optional Fields**: Non-critical statistics are optional to handle incomplete data

#### Models:
- **`ResortResponse`**: Complete ski resort information
- **`LocationBlock`**: Geographical and administrative location data
- **`AltitudeBlock`**: Elevation information for the resort
- **`SkiAreaBlock`**: Ski area operational information
- **`LiftResponse`**: Individual lift information
- **`SlopeResponse`**: Individual slope information

## Usage Examples

### Database Operations

```rust
use openslope_api::models::db::ResortRow;
use sqlx::Row;

// Fetch a resort from the database
let resort: ResortRow = sqlx::query_as!(ResortRow, 
    "SELECT * FROM resorts WHERE id = ?", resort_id)
    .fetch_one(&pool)
    .await?;

// Fetch all lifts for a resort
let lifts: Vec<LiftRow> = sqlx::query_as!(LiftRow,
    "SELECT * FROM lifts WHERE resort_id = ?", resort_id)
    .fetch_all(&pool)
    .await?;
```

### API Response Creation

```rust
use openslope_api::models::resort::{ResortResponse, LocationBlock, AltitudeBlock};

// Create a complete resort response
let resort_response = ResortResponse {
    id: "resort_123".to_string(),
    name: "Example Ski Resort".to_string(),
    location: LocationBlock {
        country: "Austria".to_string(),
        region: "Tyrol".to_string(),
        continent: "Europe".to_string(),
        latitude: 47.2628,
        longitude: 11.3936,
    },
    altitude: AltitudeBlock {
        village_altitude_m: 1200,
        min_altitude_m: 1100,
        max_altitude_m: 2500,
    },
    ski_area: SkiAreaBlock {
        name: "Example Ski Area".to_string(),
        area_type: "Alpine".to_string(),
        total_slope_km: Some(50.5),
        total_lifts: Some(15),
        snowmaking_percent: Some(80),
        night_skiing: Some(false),
    },
    lifts: vec![],
    slopes: vec![],
};

// Serialize to JSON for API response
let json_response = serde_json::to_string(&resort_response)?;
```

## Data Flow Architecture

```
Database Tables
       ↓ (SQLx FromRow)
Database Models (db.rs)
       ↓ (Business Logic)
API Response Models (resort.rs)
       ↓ (Serde Serialize)
JSON API Responses
```

## Field Reference

### Resort Information

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `id` | `String` | Unique resort identifier | ✅ |
| `name` | `String` | Official resort name | ✅ |
| `country` | `Option<String>` | Country location | ❌ |
| `region` | `Option<String>` | Administrative region | ❌ |
| `continent` | `Option<String>` | Continent location | ❌ |
| `latitude` | `Option<f64>` | Geographic latitude (WGS84) | ❌ |
| `longitude` | `Option<f64>` | Geographic longitude (WGS84) | ❌ |
| `village_altitude_m` | `Option<i32>` | Base area elevation (meters) | ❌ |
| `min_altitude_m` | `Option<i32>` | Minimum skiable elevation | ❌ |
| `max_altitude_m` | `Option<i32>` | Maximum skiable elevation | ❌ |

### Lift Information

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `id` | `i64` | Database identifier | ✅ |
| `resort_id` | `String` | Foreign key to resort | ✅ |
| `name` | `Option<String>` | Lift name | ❌ |
| `lift_type` | `Option<String>` | Type of lift | ❌ |

### Slope Information

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `id` | `i64` | Database identifier | ✅ |
| `resort_id` | `String` | Foreign key to resort | ✅ |
| `name` | `Option<String>` | Slope name | ❌ |
| `difficulty` | `Option<String>` | Difficulty level | ❌ |

## Design Principles

### Separation of Concerns
- Database models handle SQL operations
- API models handle JSON serialization
- Clear separation prevents data leakage between layers

### Type Safety
- All fields use appropriate Rust types
- Optional fields handle incomplete data gracefully
- Compile-time type checking prevents runtime errors

### Extensibility
- Models can be extended without breaking existing code
- New fields can be added as optional to maintain backward compatibility
- Clear documentation for future maintainers

## Future Extensions

The models directory is designed to be easily extensible. Future additions might include:

- **User Models**: Authentication and user management
- **Real-time Data**: Lift and slope status updates
- **Analytics Models**: Usage statistics and reporting
- **Weather Models**: Weather data integration
- **Booking Models**: Reservation and ticketing systems

## Testing

Models should be tested for:
- Database mapping accuracy
- JSON serialization/deserialization
- Field validation and constraints
- Edge cases with optional fields

## Contributing

When adding new models:
1. Follow the existing naming conventions
2. Add comprehensive documentation
3. Include usage examples
4. Update this README with new model descriptions
5. Ensure backward compatibility where possible

## Dependencies

- **serde**: For JSON serialization
- **sqlx**: For database row mapping
- **serde_json**: For JSON operations

## License

This documentation and code is part of the OpenSlope project.

---

*Last updated: March 2026*
*Version: 1.0.0*