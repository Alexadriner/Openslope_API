// API Response Transformers
// These modules transform full API responses into specific data formats
// based on what the client needs for lists of resorts

/**
 * Extracts only resort names and basic identifiers from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Simplified array with only names and IDs
 */
export function transformToNamesOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToNamesOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => ({
    id: resort.id,
    name: resort.name,
    country: resort.geography?.country ?? resort.country
  }));
}

/**
 * Extracts resort coordinates and basic location info from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Array with only coordinate information
 */
export function transformToCoordinatesOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToCoordinatesOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => {
    const latitude = resort.geography?.coordinates?.latitude ?? resort.latitude;
    const longitude = resort.geography?.coordinates?.longitude ?? resort.longitude;
    
    return {
      id: resort.id,
      name: resort.name,
      coordinates: {
        latitude: latitude,
        longitude: longitude
      },
      country: resort.geography?.country ?? resort.country,
      region: resort.geography?.region ?? resort.region
    };
  }).filter(resort => resort.coordinates.latitude != null && resort.coordinates.longitude != null);
}

/**
 * Extracts slope information with difficulty and status from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Array with only slope information
 */
export function transformToSlopesOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToSlopesOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => ({
    id: resort.id,
    name: resort.name,
    slopes: (resort.slopes ?? []).map(slope => ({
      id: slope.id,
      name: slope.name ?? "Unknown",
      difficulty: slope.difficulty ?? slope.display?.difficulty ?? "unknown",
      status: slope.status?.operational_status ?? slope.operational_status ?? "unknown",
      grooming: slope.status?.grooming_status ?? slope.grooming_status ?? "unknown",
      geometry: slope.geometry
    }))
  }));
}

/**
 * Extracts lift information with type and status from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Array with only lift information
 */
export function transformToLiftsOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToLiftsOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => ({
    id: resort.id,
    name: resort.name,
    lifts: (resort.lifts ?? []).map(lift => ({
      id: lift.id,
      name: lift.name ?? "Unnamed",
      type: lift.lift_type ?? lift.display?.lift_type ?? "unknown",
      status: lift.status?.operational_status ?? lift.operational_status ?? "unknown",
      geometry: lift.geometry
    }))
  }));
}

/**
 * Extracts live status information from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Array with only live status information
 */
export function transformToLiveStatusOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToLiveStatusOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => ({
    id: resort.id,
    name: resort.name,
    live_status: resort.live_status ?? {},
    slopes_open_count: resort.live_status?.slopes_open_count,
    lifts_open_count: resort.live_status?.lifts_open_count
  }));
}

/**
 * Extracts altitude information from an array of resorts
 * @param {Array} data - Array of full resort objects
 * @returns {Array} - Array with only altitude information
 */
export function transformToAltitudeOnly(data) {
  if (!Array.isArray(data)) {
    console.warn('transformToAltitudeOnly expects an array of resorts');
    return [];
  }
  
  return data.map(resort => ({
    id: resort.id,
    name: resort.name,
    altitude: {
      village_m: resort.altitude?.village_m ?? resort.village_altitude_m,
      min_m: resort.altitude?.min_m ?? resort.min_altitude_m,
      max_m: resort.altitude?.max_m ?? resort.max_altitude_m
    }
  }));
}

/**
 * Available transformation types for list operations
 */
export const TRANSFORMATION_TYPES = {
  NAMES_ONLY: 'names_only',
  COORDINATES_ONLY: 'coordinates_only',
  SLOPES_ONLY: 'slopes_only',
  LIFTS_ONLY: 'lifts_only',
  LIVE_STATUS_ONLY: 'live_status_only',
  ALTITUDE_ONLY: 'altitude_only'
};

/**
 * Map transformation types to their respective functions
 */
export const TRANSFORMATION_MAP = {
  [TRANSFORMATION_TYPES.NAMES_ONLY]: transformToNamesOnly,
  [TRANSFORMATION_TYPES.COORDINATES_ONLY]: transformToCoordinatesOnly,
  [TRANSFORMATION_TYPES.SLOPES_ONLY]: transformToSlopesOnly,
  [TRANSFORMATION_TYPES.LIFTS_ONLY]: transformToLiftsOnly,
  [TRANSFORMATION_TYPES.LIVE_STATUS_ONLY]: transformToLiveStatusOnly,
  [TRANSFORMATION_TYPES.ALTITUDE_ONLY]: transformToAltitudeOnly
};

/**
 * Get the appropriate transformation function based on type
 * @param {string} type - Transformation type
 * @returns {Function} - Transformation function
 */
export function getTransformationFunction(type) {
  return TRANSFORMATION_MAP[type] || null;
}

// ========================================
// Specialized Models for Map and other use cases
// ========================================

/**
 * Transform slopes data for map visualization
 * Extracts only the essential geometry and styling information needed for the map
 * @param {Array} slopes - Array of slope objects
 * @returns {Array} - Simplified slopes for map rendering
 */
export function transformSlopesForMap(slopes) {
  if (!Array.isArray(slopes)) {
    return [];
  }
  
  return slopes.map(slope => ({
    id: slope.id,
    name: slope.name ?? "Unknown",
    difficulty: slope.difficulty ?? slope.display?.difficulty ?? "unknown",
    status: slope.status?.operational_status ?? slope.operational_status ?? "unknown",
    grooming: slope.status?.grooming_status ?? slope.grooming_status ?? "unknown",
    geometry: slope.geometry,
    color: getSlopeColor(slope.difficulty ?? slope.display?.difficulty)
  })).filter(slope => slope.geometry); // Only include slopes with geometry data
}

/**
 * Transform lifts data for map visualization
 * Extracts only the essential geometry and styling information needed for the map
 * @param {Array} lifts - Array of lift objects
 * @returns {Array} - Simplified lifts for map rendering
 */
export function transformLiftsForMap(lifts) {
  if (!Array.isArray(lifts)) {
    return [];
  }
  
  return lifts.map(lift => ({
    id: lift.id,
    name: lift.name ?? "Unnamed",
    type: lift.lift_type ?? lift.display?.lift_type ?? "unknown",
    status: lift.status?.operational_status ?? lift.operational_status ?? "unknown",
    geometry: lift.geometry
  })).filter(lift => lift.geometry); // Only include lifts with geometry data
}

/**
 * Transform resorts data specifically for map markers
 * Extracts only coordinate and basic info needed for map markers
 * @param {Array} resorts - Array of resort objects
 * @returns {Array} - Simplified resorts for map markers
 */
export function transformResortsForMap(resorts) {
  if (!Array.isArray(resorts)) {
    return [];
  }
  
  return resorts.map(resort => {
    const latitude = resort.geography?.coordinates?.latitude ?? resort.latitude;
    const longitude = resort.geography?.coordinates?.longitude ?? resort.longitude;
    
    if (latitude == null || longitude == null) {
      return null;
    }
    
    return {
      id: resort.id,
      name: resort.name,
      coordinates: {
        latitude: latitude,
        longitude: longitude
      },
      country: resort.geography?.country ?? resort.country,
      region: resort.geography?.region ?? resort.region,
      popupHtml: createResortPopupHtml(resort)
    };
  }).filter(resort => resort !== null);
}

/**
 * Get color for slope difficulty (used in map rendering)
 * @param {string} difficulty - Slope difficulty level
 * @returns {string} - CSS color value
 */
function getSlopeColor(difficulty) {
  const key = String(difficulty ?? "").toLowerCase().trim();

  if (key === "green") return "#27ae60";
  if (key === "blue") return "#2980b9";
  if (key === "red") return "#c0392b";
  if (key === "black") return "#2c3e50";

  return "#7f8c8d";
}

/**
 * Create popup HTML for resort markers on map
 * @param {Object} resort - Resort object
 * @returns {string} - HTML string for popup
 */
function createResortPopupHtml(resort) {
  const resortName = resort.name ?? "Unknown resort";
  const detailHref = `/resort/${encodeURIComponent(resortName)}`;

  return (
    `<strong>${resortName}</strong><br/>` +
    `${resort.geography?.country ?? resort.country ?? "N/A"}<br/>` +
    `click <a href="${detailHref}">here</a> for more information`
  );
}

/**
 * Available specialized transformation types for map and specific use cases
 */
export const SPECIALIZED_TRANSFORMATION_TYPES = {
  SLOPES_FOR_MAP: 'slopes_for_map',
  LIFTS_FOR_MAP: 'lifts_for_map',
  RESORTS_FOR_MAP: 'resorts_for_map'
};

/**
 * Map specialized transformation types to their respective functions
 */
export const SPECIALIZED_TRANSFORMATION_MAP = {
  [SPECIALIZED_TRANSFORMATION_TYPES.SLOPES_FOR_MAP]: transformSlopesForMap,
  [SPECIALIZED_TRANSFORMATION_TYPES.LIFTS_FOR_MAP]: transformLiftsForMap,
  [SPECIALIZED_TRANSFORMATION_TYPES.RESORTS_FOR_MAP]: transformResortsForMap
};

/**
 * Get the appropriate specialized transformation function based on type
 * @param {string} type - Specialized transformation type
 * @returns {Function} - Transformation function
 */
export function getSpecializedTransformationFunction(type) {
  return SPECIALIZED_TRANSFORMATION_MAP[type] || null;
}