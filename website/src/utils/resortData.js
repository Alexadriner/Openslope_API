function titleCase(value) {
  return String(value ?? "")
    .replace(/_/g, " ")
    .trim()
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

export function normalizeResortName(value) {
  return String(value ?? "")
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[\u2010-\u2015]/g, "-")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
}

export function safeDecode(value) {
  try {
    return decodeURIComponent(value ?? "");
  } catch {
    return value ?? "";
  }
}

export function formatStatusLabel(value, fallback = "Unknown") {
  const normalized = String(value ?? "").trim();
  return normalized ? titleCase(normalized) : fallback;
}

export function getPrimaryPlace(entity) {
  return entity?.places?.[0] ?? null;
}

export function getLocationParts(entity) {
  const place = getPrimaryPlace(entity);
  if (!place) {
    return [];
  }

  return [place.locality, place.region_name, place.country_name].filter(Boolean);
}

export function getLocationLabel(entity, fallback = "Location pending") {
  const parts = getLocationParts(entity);
  return parts.length > 0 ? parts.join(", ") : fallback;
}

export function getCountryLabel(entity, fallback = "Unknown region") {
  const place = getPrimaryPlace(entity);
  return place?.country_name ?? fallback;
}

export function getRegionLabel(entity, fallback = null) {
  const place = getPrimaryPlace(entity);
  return place?.region_name ?? fallback;
}

export function getActivities(entity) {
  const raw = entity?.activities;

  if (Array.isArray(raw)) {
    return raw.filter((item) => typeof item === "string" && item.trim());
  }

  if (raw && typeof raw === "object") {
    return Object.entries(raw)
      .filter(([, enabled]) => Boolean(enabled))
      .map(([activity]) => titleCase(activity));
  }

  if (typeof raw === "string" && raw.trim()) {
    return [raw.trim()];
  }

  return [];
}

export function getCoordinatesFromGeometry(geometry) {
  if (!geometry || typeof geometry !== "object") {
    return null;
  }

  if (geometry.type === "Point" && Array.isArray(geometry.coordinates) && geometry.coordinates.length >= 2) {
    const [longitude, latitude] = geometry.coordinates;
    if (Number.isFinite(latitude) && Number.isFinite(longitude)) {
      return [latitude, longitude];
    }
  }

  const pairs = [];
  collectCoordinatePairs(geometry.coordinates, pairs);

  if (pairs.length > 0) {
    const total = pairs.reduce(
      (accumulator, [longitude, latitude]) => ({
        latitude: accumulator.latitude + latitude,
        longitude: accumulator.longitude + longitude,
      }),
      { latitude: 0, longitude: 0 }
    );

    return [total.latitude / pairs.length, total.longitude / pairs.length];
  }

  return null;
}

function collectCoordinatePairs(value, target) {
  if (!Array.isArray(value)) {
    return;
  }

  if (value.length >= 2 && Number.isFinite(value[0]) && Number.isFinite(value[1])) {
    target.push([value[0], value[1]]);
    return;
  }

  value.forEach((entry) => collectCoordinatePairs(entry, target));
}

export function getResortCoordinates(resort) {
  return getCoordinatesFromGeometry(resort?.geometry);
}

export function getOpenMetric(openCount, totalCount, fallbackTotal) {
  if (openCount == null && totalCount == null && fallbackTotal == null) {
    return "No live data";
  }

  const total = totalCount ?? fallbackTotal;
  if (openCount == null) {
    return total == null ? "No live data" : `${total} total`;
  }

  return total == null ? `${openCount} open` : `${openCount}/${total} open`;
}

export function getResortStats(resort) {
  const stats = resort?.stats ?? {};
  const snapshot = resort?.latest_snapshot ?? {};
  const lifts = resort?.lifts ?? [];
  const slopes = resort?.slopes ?? [];

  return {
    liftCount: stats.lift_count ?? lifts.length,
    slopeCount: stats.slope_count ?? slopes.length,
    openLiftCount: stats.open_lift_count ?? snapshot.lifts_open_count ?? null,
    openSlopeCount: stats.open_slope_count ?? snapshot.slopes_open_count ?? null,
    totalLiftCount: snapshot.lifts_total_count ?? stats.lift_count ?? lifts.length,
    totalSlopeCount: snapshot.slopes_total_count ?? stats.slope_count ?? slopes.length,
  };
}

export function getSlopeDifficultyClass(value) {
  const normalized = String(value ?? "").trim().toLowerCase();
  return normalized || "unknown";
}

export function getWebsites(entity) {
  const websites = entity?.websites;

  if (Array.isArray(websites)) {
    return websites
      .map((entry) => (typeof entry === "string" ? { label: "Website", url: entry } : entry))
      .filter((entry) => entry?.url);
  }

  if (websites && typeof websites === "object") {
    return Object.entries(websites)
      .filter(([, url]) => typeof url === "string" && url.trim())
      .map(([label, url]) => ({ label: titleCase(label), url }));
  }

  if (typeof websites === "string" && websites.trim()) {
    return [{ label: "Website", url: websites }];
  }

  return [];
}
