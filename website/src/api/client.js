import { cacheMiddleware, getCachedResponse, cacheResponse, getCacheStats, clearCache } from './cache.js';
import { getTransformationFunction, getSpecializedTransformationFunction } from './models/index.js';

const API_BASE = "http://localhost:8080";
const FALLBACK_API_KEY = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA";

function normalizePath(path) {
  if (!path) {
    return "/";
  }
  return path.startsWith("/") ? path : `/${path}`;
}

/**
 * Enhanced API fetch function with caching and transformation support
 * @param {string} path - API endpoint path
 * @param {Object} options - Request options
 * @returns {Promise} - API response
 */
export async function apiFetch(path, options = {}) {
  const url = new URL(`${API_BASE}${normalizePath(path)}`);
  const apiKey = options.apiKey || localStorage.getItem("apiKey") || FALLBACK_API_KEY;
  const method = String(options.method || "GET").toUpperCase();
  const transformation = options.transformation;
  const specializedTransformation = options.specializedTransformation;
  const cacheTTL = options.cacheTTL;

  // Add API key to query parameters for GET requests
  if (apiKey && method === "GET") {
    url.searchParams.set("api_key", apiKey);
  }

  // Prepare fetch options
  const { apiKey: _, transformation: __, specializedTransformation: ___, cacheTTL: ____, ...fetchOptions } = options;
  
  // Check cache first (only for GET requests)
  if (method === "GET") {
    const cachedData = getCachedResponse(url.toString(), { apiKey, transformation, specializedTransformation });
    if (cachedData) {
      return cachedData;
    }
  }

  // Make the actual API request
  const res = await fetch(url.toString(), {
    headers: {
      "Content-Type": "application/json",
      ...(apiKey && method !== "GET" ? { Authorization: `Bearer ${apiKey}` } : {}),
      ...(fetchOptions.headers || {}),
    },
    method,
    ...fetchOptions,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "API error");
  }

  // Parse response
  const contentType = res.headers.get("content-type") || "";
  let data;
  
  if (contentType.includes("application/json")) {
    data = await res.json();
  } else {
    data = await res.text();
  }

  // Apply transformations if requested
  let transformedData = data;

  if (transformation) {
    const transformFunction = getTransformationFunction(transformation);
    if (transformFunction) {
      transformedData = transformFunction(data);
    }
  } else if (specializedTransformation) {
    const transformFunction = getSpecializedTransformationFunction(specializedTransformation);
    if (transformFunction) {
      transformedData = transformFunction(data);
    }
  }

  // Cache the transformed data
  if (method === "GET") {
    const ttl = cacheTTL || (transformation || specializedTransformation ? 10 * 60 * 1000 : 5 * 60 * 1000); // 10 min for transformed, 5 min for full data
    cacheResponse(url.toString(), { apiKey, transformation, specializedTransformation }, transformedData, ttl);
  }

  return transformedData;
}

/**
 * Fetch resorts with optional transformation
 * @param {string} transformation - Type of transformation to apply
 * @param {Object} options - Additional request options
 * @returns {Promise} - Transformed resort data
 */
export async function fetchResorts(transformation = null, options = {}) {
  return apiFetch("/resorts", {
    ...options,
    transformation
  });
}

/**
 * Fetch resorts specifically for map rendering
 * @param {Object} options - Request options
 * @returns {Promise} - Resorts data optimized for map
 */
export async function fetchResortsForMap(options = {}) {
  return apiFetch("/resorts", {
    ...options,
    specializedTransformation: "resorts_for_map"
  });
}

/**
 * Fetch slopes data with optional transformation
 * @param {string} transformation - Type of transformation to apply
 * @param {Object} options - Additional request options
 * @returns {Promise} - Transformed slopes data
 */
export async function fetchSlopes(transformation = null, options = {}) {
  return apiFetch("/slopes", {
    ...options,
    transformation
  });
}

/**
 * Fetch slopes specifically for map rendering
 * @param {Object} options - Request options
 * @returns {Promise} - Slopes data optimized for map
 */
export async function fetchSlopesForMap(options = {}) {
  return apiFetch("/slopes", {
    ...options,
    specializedTransformation: "slopes_for_map"
  });
}

/**
 * Fetch lifts data with optional transformation
 * @param {string} transformation - Type of transformation to apply
 * @param {Object} options - Additional request options
 * @returns {Promise} - Transformed lifts data
 */
export async function fetchLifts(transformation = null, options = {}) {
  return apiFetch("/lifts", {
    ...options,
    transformation
  });
}

/**
 * Fetch lifts specifically for map rendering
 * @param {Object} options - Request options
 * @returns {Promise} - Lifts data optimized for map
 */
export async function fetchLiftsForMap(options = {}) {
  return apiFetch("/lifts", {
    ...options,
    specializedTransformation: "lifts_for_map"
  });
}

/**
 * Fetch a specific resort by ID
 * @param {string|number} id - Resort ID
 * @param {Object} options - Request options
 * @returns {Promise} - Resort data
 */
export async function fetchResort(id, options = {}) {
  return apiFetch(`/resorts/${id}`, options);
}

/**
 * Check if data is cached
 * @param {string} path - API endpoint path
 * @param {Object} options - Request options
 * @returns {boolean} - True if cached and valid
 */
export function isDataCached(path, options = {}) {
  const url = new URL(`${API_BASE}${normalizePath(path)}`);
  const apiKey = options.apiKey || localStorage.getItem("apiKey") || FALLBACK_API_KEY;
  const method = String(options.method || "GET").toUpperCase();
  const transformation = options.transformation;
  const specializedTransformation = options.specializedTransformation;

  if (method !== "GET") {
    return false;
  }

  return !!getCachedResponse(url.toString(), { apiKey, transformation, specializedTransformation });
}

/**
 * Clear cache for a specific endpoint
 * @param {string} path - API endpoint path
 * @param {Object} options - Request options
 */
export function clearCacheForEndpoint(path, options = {}) {
  const url = new URL(`${API_BASE}${normalizePath(path)}`);
  const apiKey = options.apiKey || localStorage.getItem("apiKey") || FALLBACK_API_KEY;
  const transformation = options.transformation;
  const specializedTransformation = options.specializedTransformation;

  // This is a simplified version - in a real implementation, you might want
  // to clear all variations of the endpoint
  // For now, we'll use the cache middleware's remove function
  const { removeCachedResponse } = require('./cache.js');
  removeCachedResponse(url.toString(), { apiKey, transformation, specializedTransformation });
}

/**
 * Get cache statistics (re-export from cache module)
 * @returns {Object} - Cache statistics
 */
export { getCacheStats } from './cache.js';

/**
 * Clear all cached responses (re-export from cache module)
 */
export { clearCache } from './cache.js';
