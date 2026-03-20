# API Caching and Models System

This document describes the new caching logic and modular API request system implemented for the Openslope website.

## Overview

The system provides two main features:

1. **Caching Logic**: Reduces API calls and improves performance by caching responses
2. **Modular API Requests**: Allows fetching only the specific data needed through transformation functions

## File Structure

```
website/src/api/
├── client.js          # Enhanced API client with caching and transformations
├── cache.js           # Caching logic and utilities
├── models/
│   └── index.js       # API response transformers and models
└── README.md          # This documentation

website/src/pages/
├── Resorts.jsx        # Updated to use names_only transformation
├── Map.jsx           # Updated to use specialized map transformations
└── ApiDemo.jsx       # Demo page for testing the new system
```

## Caching System

### Features

- **In-memory caching** with configurable TTL (Time To Live)
- **Automatic cleanup** of expired entries
- **Cache size limits** (max 100 entries by default)
- **Cache statistics** and monitoring
- **Cache invalidation** for specific endpoints

### Configuration

```javascript
const CACHE_CONFIG = {
  DEFAULT_TTL: 5 * 60 * 1000,    // 5 minutes default
  MAX_CACHE_SIZE: 100,           // Maximum 100 cache entries
  KEY_PREFIX: 'openslope_api_cache_'
};
```

### Usage

```javascript
import { 
  fetchResorts, 
  isDataCached, 
  clearCacheForEndpoint, 
  getCacheStats 
} from '../api/client';

// Fetch data (automatically cached)
const resorts = await fetchResorts('names_only');

// Check if data is cached
const isCached = isDataCached('/resorts', { transformation: 'names_only' });

// Clear cache for specific endpoint
clearCacheForEndpoint('/resorts');

// Get cache statistics
const stats = getCacheStats();
```

## API Response Transformers

### Available Transformations

#### General Transformations (for arrays of resorts)
- `names_only` - Only resort names and IDs
- `coordinates_only` - Only coordinate information
- `slopes_only` - Only slope information
- `lifts_only` - Only lift information
- `live_status_only` - Only live status information
- `altitude_only` - Only altitude information

#### Specialized Transformations (for specific use cases)
- `resorts_for_map` - Resorts optimized for map rendering
- `slopes_for_map` - Slopes optimized for map rendering
- `lifts_for_map` - Lifts optimized for map rendering

### Usage Examples

```javascript
import { 
  fetchResorts, 
  fetchResortsForMap,
  TRANSFORMATION_TYPES 
} from '../api/client';

// Fetch only names (reduced bandwidth)
const resortNames = await fetchResorts(TRANSFORMATION_TYPES.NAMES_ONLY);

// Fetch resorts optimized for map
const mapResorts = await fetchResortsForMap();

// Fetch with custom options
const coords = await fetchResorts(TRANSFORMATION_TYPES.COORDINATES_ONLY, {
  cacheTTL: 10 * 60 * 1000  // 10 minutes cache
});
```

## Benefits

### Performance Improvements
- **Reduced bandwidth usage**: Only fetch the data you need
- **Faster response times**: Cached responses for repeated requests
- **Optimized rendering**: Specialized data structures for maps

### Developer Experience
- **Easy to use**: Simple API with clear transformation types
- **Flexible**: Easy to add new transformation types
- **Type-safe**: Clear function signatures and return types
- **Debugging**: Cache statistics and monitoring tools

### Use Cases

#### 1. Resort List Page
```javascript
// Before: Fetches all resort data (large payload)
const resorts = await apiFetch('/resorts');

// After: Fetches only names and IDs (small payload)
const resortNames = await fetchResorts('names_only');
```

#### 2. Map Page
```javascript
// Before: Fetches full data and processes it client-side
const resorts = await apiFetch('/resorts');

// After: Fetches pre-processed data optimized for maps
const mapResorts = await fetchResortsForMap();
```

#### 3. Specific Data Needs
```javascript
// Need only coordinates for distance calculations
const coords = await fetchResorts('coordinates_only');

// Need only slopes for difficulty analysis
const slopes = await fetchResorts('slopes_only');
```

## Implementation Details

### Cache Key Generation

Cache keys are generated based on:
- API endpoint URL
- HTTP method
- API key
- Transformation type
- Specialized transformation type

This ensures that different transformations of the same endpoint are cached separately.

### Cache Expiration

- **Default TTL**: 5 minutes for full data
- **Transformed data TTL**: 10 minutes (longer since less likely to change)
- **Automatic cleanup**: Runs every minute to remove expired entries

### Memory Management

- **Size limits**: Maximum 100 cache entries
- **LRU eviction**: Removes oldest entries when limit is reached
- **Memory efficient**: Only stores transformed data, not full responses

## Testing

Use the `ApiDemo` page to test the new system:

1. Navigate to `/api-demo`
2. Test different transformation types
3. Monitor cache statistics
4. Verify caching behavior
5. Test cache invalidation

## Migration Guide

### For Existing Pages

1. **Resorts List**: Already updated to use `names_only` transformation
2. **Map Page**: Already updated to use specialized map transformations
3. **Other Pages**: Update as needed based on data requirements

### For New Development

1. **Assess data needs**: Determine what data is actually required
2. **Choose transformation**: Select appropriate transformation type
3. **Use new functions**: Use `fetchResorts`, `fetchSlopes`, etc.
4. **Monitor performance**: Check cache statistics and response times

## Future Enhancements

- **Persistent caching**: Store cache in localStorage for longer persistence
- **Cache warming**: Pre-fetch commonly used data
- **Advanced statistics**: More detailed cache performance metrics
- **Cache invalidation strategies**: Smart invalidation based on data changes
- **Compression**: Compress cached data for memory efficiency

## Troubleshooting

### Common Issues

1. **Cache not working**: Check that you're using GET requests (only GET requests are cached)
2. **Wrong data**: Verify transformation type matches your needs
3. **Performance issues**: Check cache statistics and adjust TTL as needed
4. **Memory usage**: Monitor cache size and adjust MAX_CACHE_SIZE if needed

### Debug Tools

```javascript
// Check cache statistics
const stats = getCacheStats();
console.log('Cache stats:', stats);

// Check if specific data is cached
const isCached = isDataCached('/resorts', { transformation: 'names_only' });
console.log('Is cached:', isCached);

// Clear all cache
clearCache();