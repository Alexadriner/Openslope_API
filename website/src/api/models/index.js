import {
  formatStatusLabel,
  getCountryLabel,
  getLocationLabel,
  getResortCoordinates,
  getResortStats,
} from "../../utils/resortData.js";

export function transformToNamesOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data.map((resort) => ({
    id: resort.id,
    name: resort.name,
    country: getCountryLabel(resort),
    location: getLocationLabel(resort),
  }));
}

export function transformToCoordinatesOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data
    .map((resort) => {
      const coordinates = getResortCoordinates(resort);
      if (!coordinates) {
        return null;
      }

      return {
        id: resort.id,
        name: resort.name,
        coordinates: {
          latitude: coordinates[0],
          longitude: coordinates[1],
        },
        country: getCountryLabel(resort),
        location: getLocationLabel(resort),
      };
    })
    .filter(Boolean);
}

export function transformToSlopesOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data.map((resort) => ({
    id: resort.id,
    name: resort.name,
    slopes: (resort.slopes ?? []).map((slope) => ({
      id: slope.id,
      name: slope.name ?? "Unnamed slope",
      difficulty: slope.difficulty ?? "unknown",
      status: slope.status ?? "unknown",
      grooming: slope.grooming ?? "unknown",
      statusLabel: formatStatusLabel(slope.status),
      geometry: slope.geometry,
    })),
  }));
}

export function transformToLiftsOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data.map((resort) => ({
    id: resort.id,
    name: resort.name,
    lifts: (resort.lifts ?? []).map((lift) => ({
      id: lift.id,
      name: lift.name ?? "Unnamed lift",
      type: lift.lift_type ?? "unknown",
      status: lift.status ?? "unknown",
      statusLabel: formatStatusLabel(lift.status),
      capacity: lift.capacity ?? null,
      duration: lift.duration ?? null,
      geometry: lift.geometry,
    })),
  }));
}

export function transformToLiveStatusOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data.map((resort) => {
    const stats = getResortStats(resort);

    return {
      id: resort.id,
      name: resort.name,
      latest_snapshot: resort.latest_snapshot ?? null,
      slopes_open_count: stats.openSlopeCount,
      lifts_open_count: stats.openLiftCount,
      slopes_total_count: stats.totalSlopeCount,
      lifts_total_count: stats.totalLiftCount,
    };
  });
}

export function transformToAltitudeOnly(data) {
  if (!Array.isArray(data)) {
    return [];
  }

  return data.map((resort) => ({
    id: resort.id,
    name: resort.name,
    altitude: null,
    location: getLocationLabel(resort),
  }));
}

export const TRANSFORMATION_TYPES = {
  NAMES_ONLY: "names_only",
  COORDINATES_ONLY: "coordinates_only",
  SLOPES_ONLY: "slopes_only",
  LIFTS_ONLY: "lifts_only",
  LIVE_STATUS_ONLY: "live_status_only",
  ALTITUDE_ONLY: "altitude_only",
};

export const TRANSFORMATION_MAP = {
  [TRANSFORMATION_TYPES.NAMES_ONLY]: transformToNamesOnly,
  [TRANSFORMATION_TYPES.COORDINATES_ONLY]: transformToCoordinatesOnly,
  [TRANSFORMATION_TYPES.SLOPES_ONLY]: transformToSlopesOnly,
  [TRANSFORMATION_TYPES.LIFTS_ONLY]: transformToLiftsOnly,
  [TRANSFORMATION_TYPES.LIVE_STATUS_ONLY]: transformToLiveStatusOnly,
  [TRANSFORMATION_TYPES.ALTITUDE_ONLY]: transformToAltitudeOnly,
};

export function getTransformationFunction(type) {
  return TRANSFORMATION_MAP[type] || null;
}

export function transformSlopesForMap(slopes) {
  if (!Array.isArray(slopes)) {
    return [];
  }

  return slopes
    .map((slope) => ({
      id: slope.id,
      name: slope.name ?? "Unnamed slope",
      difficulty: slope.difficulty ?? "unknown",
      status: slope.status ?? "unknown",
      grooming: slope.grooming ?? "unknown",
      geometry: slope.geometry,
    }))
    .filter((slope) => slope.geometry);
}

export function transformLiftsForMap(lifts) {
  if (!Array.isArray(lifts)) {
    return [];
  }

  return lifts
    .map((lift) => ({
      id: lift.id,
      name: lift.name ?? "Unnamed lift",
      type: lift.lift_type ?? "unknown",
      status: lift.status ?? "unknown",
      capacity: lift.capacity ?? null,
      duration: lift.duration ?? null,
      geometry: lift.geometry,
    }))
    .filter((lift) => lift.geometry);
}

export function transformResortsForMap(resorts) {
  if (!Array.isArray(resorts)) {
    return [];
  }

  return resorts
    .map((resort) => {
      const coordinates = getResortCoordinates(resort);
      if (!coordinates) {
        return null;
      }

      return {
        ...resort,
        popupHtml: createResortPopupHtml(resort),
        coordinates: {
          latitude: coordinates[0],
          longitude: coordinates[1],
        },
      };
    })
    .filter(Boolean);
}

function createResortPopupHtml(resort) {
  const resortName = resort.name ?? "Unknown resort";
  const detailHref = `/resort/${encodeURIComponent(resortName)}`;

  return (
    `<strong>${resortName}</strong><br/>` +
    `${getLocationLabel(resort)}<br/>` +
    `Open lifts: ${getResortStats(resort).openLiftCount ?? "?"}<br/>` +
    `<a href="${detailHref}">Open resort page</a>`
  );
}

export const SPECIALIZED_TRANSFORMATION_TYPES = {
  SLOPES_FOR_MAP: "slopes_for_map",
  LIFTS_FOR_MAP: "lifts_for_map",
  RESORTS_FOR_MAP: "resorts_for_map",
};

export const SPECIALIZED_TRANSFORMATION_MAP = {
  [SPECIALIZED_TRANSFORMATION_TYPES.SLOPES_FOR_MAP]: transformSlopesForMap,
  [SPECIALIZED_TRANSFORMATION_TYPES.LIFTS_FOR_MAP]: transformLiftsForMap,
  [SPECIALIZED_TRANSFORMATION_TYPES.RESORTS_FOR_MAP]: transformResortsForMap,
};

export function getSpecializedTransformationFunction(type) {
  return SPECIALIZED_TRANSFORMATION_MAP[type] || null;
}
