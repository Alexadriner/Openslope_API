// API Response Cache
// Implements caching logic for API responses to improve performance
// and reduce unnecessary network requests

/**
 * Cache configuration
 */
const CACHE_CONFIG = {
  // Default cache duration: 5 minutes
  DEFAULT_TTL: 5 * 60 * 1000,
  // Maximum cache size: 100 entries
  MAX_CACHE_SIZE: 100,
  // Cache key prefix
  KEY_PREFIX: 'openslope_api_cache_'
};

/**
 * Cache entry structure
 */
class CacheEntry {
  constructor(data, timestamp, ttl) {
    this.data = data;
    this.timestamp = timestamp;
    this.ttl = ttl;
  }

  isExpired() {
    return Date.now() - this.timestamp > this.ttl;
  }
}

/**
 * Simple in-memory cache implementation
 */
class ApiCache {
  constructor() {
    this.cache = new Map();
    this.cleanupInterval = null;
    this.startCleanup();
  }

  /**
   * Start periodic cleanup of expired cache entries
   */
  startCleanup() {
    if (this.cleanupInterval) return;

    this.cleanupInterval = setInterval(() => {
      this.cleanup();
    }, 60000); // Cleanup every minute
  }

  /**
   * Stop cleanup interval
   */
  stopCleanup() {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
  }

  /**
   * Generate cache key from URL and options
   * @param {string} url - API endpoint URL
   * @param {Object} options - Request options
   * @returns {string} - Cache key
   */
  generateKey(url, options = {}) {
    const { method = 'GET', apiKey, transformation } = options;
    const keyData = {
      url,
      method: method.toUpperCase(),
      apiKey: apiKey || null,
      transformation: transformation || null
    };
    
    return CACHE_CONFIG.KEY_PREFIX + JSON.stringify(keyData);
  }

  /**
   * Store data in cache
   * @param {string} key - Cache key
   * @param {any} data - Data to cache
   * @param {number} ttl - Time to live in milliseconds
   */
  set(key, data, ttl = CACHE_CONFIG.DEFAULT_TTL) {
    // Remove oldest entries if cache is full
    if (this.cache.size >= CACHE_CONFIG.MAX_CACHE_SIZE) {
      const oldestKey = this.cache.keys().next().value;
      this.cache.delete(oldestKey);
    }

    this.cache.set(key, new CacheEntry(data, Date.now(), ttl));
  }

  /**
   * Get data from cache
   * @param {string} key - Cache key
   * @returns {any|null} - Cached data or null if not found/expired
   */
  get(key) {
    const entry = this.cache.get(key);
    
    if (!entry) {
      return null;
    }

    if (entry.isExpired()) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Check if key exists in cache and is not expired
   * @param {string} key - Cache key
   * @returns {boolean} - True if cached and valid
   */
  has(key) {
    return this.get(key) !== null;
  }

  /**
   * Remove entry from cache
   * @param {string} key - Cache key
   */
  delete(key) {
    this.cache.delete(key);
  }

  /**
   * Clear all cache entries
   */
  clear() {
    this.cache.clear();
  }

  /**
   * Remove expired entries from cache
   */
  cleanup() {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (entry.isExpired()) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Get cache statistics
   * @returns {Object} - Cache statistics
   */
  getStats() {
    const now = Date.now();
    let validCount = 0;
    let expiredCount = 0;

    for (const entry of this.cache.values()) {
      if (entry.isExpired()) {
        expiredCount++;
      } else {
        validCount++;
      }
    }

    return {
      total: this.cache.size,
      valid: validCount,
      expired: expiredCount,
      maxSize: CACHE_CONFIG.MAX_CACHE_SIZE
    };
  }
}

// Create global cache instance
const apiCache = new ApiCache();

/**
 * Cache API response
 * @param {string} url - API endpoint URL
 * @param {Object} options - Request options
 * @param {any} data - Data to cache
 * @param {number} ttl - Time to live in milliseconds
 */
export function cacheResponse(url, options, data, ttl) {
  const key = apiCache.generateKey(url, options);
  apiCache.set(key, data, ttl);
}

/**
 * Get cached API response
 * @param {string} url - API endpoint URL
 * @param {Object} options - Request options
 * @returns {any|null} - Cached data or null if not found/expired
 */
export function getCachedResponse(url, options) {
  const key = apiCache.generateKey(url, options);
  return apiCache.get(key);
}

/**
 * Check if response is cached and valid
 * @param {string} url - API endpoint URL
 * @param {Object} options - Request options
 * @returns {boolean} - True if cached and valid
 */
export function isResponseCached(url, options) {
  return apiCache.has(apiCache.generateKey(url, options));
}

/**
 * Remove cached response
 * @param {string} url - API endpoint URL
 * @param {Object} options - Request options
 */
export function removeCachedResponse(url, options) {
  const key = apiCache.generateKey(url, options);
  apiCache.delete(key);
}

/**
 * Get cache statistics
 * @returns {Object} - Cache statistics
 */
export function getCacheStats() {
  return apiCache.getStats();
}

/**
 * Clear all cached responses
 */
export function clearCache() {
  apiCache.clear();
}

/**
 * Cache middleware for API requests
 * Automatically caches GET requests and checks cache before making requests
 */
export const cacheMiddleware = {
  /**
   * Check cache before making request
   * @param {string} url - API endpoint URL
   * @param {Object} options - Request options
   * @returns {any|null} - Cached data or null
   */
  beforeRequest: (url, options) => {
    // Only cache GET requests
    if (options.method && options.method.toUpperCase() !== 'GET') {
      return null;
    }

    return getCachedResponse(url, options);
  },

  /**
   * Cache response after successful request
   * @param {string} url - API endpoint URL
   * @param {Object} options - Request options
   * @param {any} data - Response data
   * @param {number} ttl - Time to live in milliseconds
   */
  afterRequest: (url, options, data, ttl) => {
    // Only cache GET requests
    if (options.method && options.method.toUpperCase() !== 'GET') {
      return;
    }

    cacheResponse(url, options, data, ttl);
  }
};

// Export the cache instance for advanced usage
export { apiCache };