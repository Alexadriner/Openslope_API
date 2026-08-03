import { useEffect, useRef, useState } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "leaflet.markercluster";
import "leaflet.markercluster/dist/MarkerCluster.css";
import "leaflet.markercluster/dist/MarkerCluster.Default.css";
import { fetchResortsForMap, fetchLiftsForResort, fetchSlopesForResort } from "../api/client";
import { formatStatusLabel, getLocationLabel, getResortStats } from "../utils/resortData";
import "../stylesheets/base.css";
import "../stylesheets/map.css";

const DEFAULT_CENTER = [46.8, 8.2];
const DEFAULT_ZOOM = 5;
const LIFTS_MIN_ZOOM = 10;
const SLOPES_MIN_ZOOM = 11;
const PAGE_SIZE = 50;

const RESORT_MARKER_ICON = L.divIcon({
  className: "single-resort-marker-icon",
  html: '<div class="single-resort-marker-dot" aria-hidden="true"></div>',
  iconSize: [30, 30],
  iconAnchor: [15, 15],
  popupAnchor: [0, -15],
});

function getSlopeColor(difficulty) {
  const key = String(difficulty ?? "").toLowerCase().trim();

  if (
    [
      "easy",
      "beginner",
      "novice",
      "blue",
      "blue_square",
      "blue square",
    ].includes(key)
  ) {
    return "#2075c7";
  }

  if (
    [
      "intermediate",
      "moderate",
      "red",
      "red_run",
      "red run",
    ].includes(key)
  ) {
    return "#c43d36";
  }

  if (
    [
      "advanced",
      "difficult",
      "very_difficult",
      "very difficult",
      "expert",
      "extreme",
      "black",
      "black_diamond",
      "black diamond",
      "double_black",
      "double black",
      "double_black_diamond",
      "double black diamond",
    ].includes(key)
  ) {
    return "#1f2933";
  }

  if (["green", "green_circle", "green circle"].includes(key)) {
    return "#30a46c";
  }

  return "#75879a";
}

function createFeatureLayer(geometry, style, popupText) {
  if (!geometry) {
    return null;
  }

  const layer = L.geoJSON(geometry, { style });
  const bounds = layer.getBounds();
  if (!bounds.isValid()) {
    return null;
  }

  if (popupText) {
    layer.bindPopup(popupText);
  }

  return { layer, bounds };
}

function createResortPopupHtml(resort) {
  const stats = getResortStats(resort);
  return `
    <strong>${resort.name}</strong><br/>
    ${getLocationLabel(resort)}<br/>
    ${getOpenMetricLine("Lifts", stats.openLiftCount, stats.totalLiftCount, stats.liftCount)}<br/>
    ${getOpenMetricLine("Slopes", stats.openSlopeCount, stats.totalSlopeCount, stats.slopeCount)}<br/>
    <a href="/resorts/${resort.id}">Open resort page</a>
  `;
}

function getOpenMetricLine(label, openCount, totalCount, fallbackTotal) {
  const total = totalCount ?? fallbackTotal;
  if (openCount == null && total == null) {
    return `${label}: no live data`;
  }
  if (openCount == null) {
    return `${label}: ${total} total`;
  }
  return `${label}: ${openCount}/${total ?? "?"} open`;
}

function createMarkerWithBasicData(resort) {
  const coordinates = resort.coordinates;
  if (!coordinates) {
    return null;
  }

  return {
    id: resort.id,
    marker: L.marker([coordinates.latitude, coordinates.longitude], {
      icon: RESORT_MARKER_ICON,
    }).bindPopup(createResortPopupHtml(resort)),
    point: L.latLng(coordinates.latitude, coordinates.longitude),
    resort: resort,
    liftsLoaded: false,
    slopesLoaded: false,
    lifts: [],
    slopes: [],
  };
}

export default function ResortMap() {
  const containerRef = useRef(null);
  const mapRef = useRef(null);
  const clusterRef = useRef(null);
  const liftsLayerRef = useRef(null);
  const slopesLayerRef = useRef(null);
  const resortsMapRef = useRef(new Map()); // Map of resort.id -> prepared data
  const loadingResortGeometryRef = useRef(new Set()); // Track which resorts are currently loading geometry

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Initialize map
  useEffect(() => {
    if (mapRef.current || !containerRef.current) {
      return undefined;
    }

    const map = L.map(containerRef.current, {
      center: DEFAULT_CENTER,
      zoom: DEFAULT_ZOOM,
      minZoom: 2,
      worldCopyJump: true,
    });

    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      maxZoom: 19,
    }).addTo(map);

    clusterRef.current = L.markerClusterGroup({
      showCoverageOnHover: false,
      spiderfyOnMaxZoom: true,
      maxClusterRadius: 55,
    }).addTo(map);

    liftsLayerRef.current = L.layerGroup().addTo(map);
    slopesLayerRef.current = L.layerGroup().addTo(map);
    mapRef.current = map;

    return () => {
      map.remove();
      mapRef.current = null;
      clusterRef.current = null;
      liftsLayerRef.current = null;
      slopesLayerRef.current = null;
      resortsMapRef.current.clear();
    };
  }, []);

  // Load resort markers (lightweight operation)
  useEffect(() => {
    let cancelled = false;

    async function loadResortMarkers() {
      if (!mapRef.current || !clusterRef.current) {
        return;
      }

      try {
        setLoading(true);
        setError("");

        let allResorts = [];
        let offset = 0;
        let hasMore = true;

        // Load resorts in pages to avoid loading all at once
        while (hasMore && !cancelled) {
          const resortsPage = await fetchResortsForMap(PAGE_SIZE, offset);
          
          if (cancelled) {
            return;
          }

          if (!Array.isArray(resortsPage) || resortsPage.length === 0) {
            hasMore = false;
            break;
          }

          allResorts = allResorts.concat(resortsPage);
          offset += PAGE_SIZE;

          // Add markers to map as we load them (don't wait for all pages)
          resortsPage.forEach((resort) => {
            const prepared = createMarkerWithBasicData(resort);
            if (prepared) {
              resortsMapRef.current.set(resort.id, prepared);
              clusterRef.current.addLayer(prepared.marker);
            }
          });

          // Only load 3 pages initially (150 resorts), remaining load on demand
          if (allResorts.length >= 150) {
            hasMore = false;
          }
        }

        if (!cancelled) {
          const allPoints = Array.from(resortsMapRef.current.values())
            .map((r) => r.point)
            .filter(Boolean);
          
          if (allPoints.length > 0) {
            const bounds = L.latLngBounds(allPoints);
            if (bounds.isValid()) {
              mapRef.current.fitBounds(bounds.pad(0.18));
            }
          }
        }
      } catch (err) {
        if (!cancelled) {
          setError(err.message || "Map data could not be loaded.");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadResortMarkers();

    return () => {
      cancelled = true;
    };
  }, []);

  // Load geometry (slopes/lifts) on demand based on zoom and viewport
  useEffect(() => {
    const map = mapRef.current;
    const liftsLayer = liftsLayerRef.current;
    const slopesLayer = slopesLayerRef.current;

    if (!map || !liftsLayer || !slopesLayer) {
      return undefined;
    }

    let geometryLoadTimeout = null;

    const refreshVisibleLayers = () => {
      if (geometryLoadTimeout) {
        clearTimeout(geometryLoadTimeout);
      }

      geometryLoadTimeout = setTimeout(() => {
        const visibleBounds = map.getBounds().pad(0.12);
        const zoom = map.getZoom();
        const shouldLoadGeometry = zoom >= LIFTS_MIN_ZOOM;

        if (!shouldLoadGeometry) {
          liftsLayer.clearLayers();
          slopesLayer.clearLayers();
          return;
        }

        // Find visible resorts and load their geometry
        const resortsToLoad = Array.from(resortsMapRef.current.values()).filter((resortData) => {
          if (visibleBounds.contains(resortData.point)) {
            // Resort is in bounds
            const geomLoading = loadingResortGeometryRef.current.has(resortData.id);
            const alreadyLoaded = resortData.liftsLoaded && resortData.slopesLoaded;
            return !geomLoading && !alreadyLoaded;
          }
          return false;
        });

        // Load geometry for visible resorts
        resortsToLoad.forEach((resortData) => {
          loadingResortGeometryRef.current.add(resortData.id);

          Promise.all([
            fetchLiftsForResort(resortData.id).catch(() => []),
            fetchSlopesForResort(resortData.id).catch(() => []),
          ])
            .then(([liftsData, slopesData]) => {
              // Create layers from fetched data
              const lifts = Array.isArray(liftsData)
                ? liftsData
                    .map((lift) =>
                      createFeatureLayer(
                        lift.geometry,
                        { color: "#6b7280", weight: 2, opacity: 0.85 },
                        `${lift.name ?? "Unnamed lift"}<br/>${formatStatusLabel(lift.status)}`
                      )
                    )
                    .filter(Boolean)
                : [];

              const slopes = Array.isArray(slopesData)
                ? slopesData
                    .map((slope) =>
                      createFeatureLayer(
                        slope.geometry,
                        {
                          color: getSlopeColor(slope.difficulty),
                          weight: 2.5,
                          opacity: 0.95,
                        },
                        `${slope.name ?? "Unnamed slope"}<br/>${formatStatusLabel(slope.difficulty)} · ${formatStatusLabel(slope.status)}`
                      )
                    )
                    .filter(Boolean)
                : [];

              resortData.lifts = lifts;
              resortData.slopes = slopes;
              resortData.liftsLoaded = true;
              resortData.slopesLoaded = true;
              loadingResortGeometryRef.current.delete(resortData.id);
            })
            .catch(() => {
              loadingResortGeometryRef.current.delete(resortData.id);
            });
        });

        // Display loaded geometry
        const showLifts = zoom >= LIFTS_MIN_ZOOM;
        const showSlopes = zoom >= SLOPES_MIN_ZOOM;

        liftsLayer.clearLayers();
        slopesLayer.clearLayers();

        Array.from(resortsMapRef.current.values()).forEach((resortData) => {
          const pointInBounds = visibleBounds.contains(resortData.point);
          if (pointInBounds && showLifts) {
            resortData.lifts.forEach((lift) => {
              if (visibleBounds.intersects(lift.bounds)) {
                liftsLayer.addLayer(lift.layer);
              }
            });
          }

          if (pointInBounds && showSlopes) {
            resortData.slopes.forEach((slope) => {
              if (visibleBounds.intersects(slope.bounds)) {
                slopesLayer.addLayer(slope.layer);
              }
            });
          }
        });
      }, 300); // Debounce to avoid too many requests
    };

    refreshVisibleLayers();
    map.on("moveend", refreshVisibleLayers);
    map.on("zoomend", refreshVisibleLayers);

    return () => {
      map.off("moveend", refreshVisibleLayers);
      map.off("zoomend", refreshVisibleLayers);
      if (geometryLoadTimeout) {
        clearTimeout(geometryLoadTimeout);
      }
    };
  }, [loading]);

  return (
    <div className="page-container map-page">
      <h1>Ski Map</h1>
      <p className="map-subtitle">
        Browse resort locations first, then zoom in for lift lines and slope geometry from the new API.
      </p>

      <div className="map-frame">
        <div ref={containerRef} className="ski-map-canvas" />
      </div>

      <p className="map-status">
        {loading ? "Loading resort map..." : "Zoom in to reveal lifts and slopes."}
      </p>
      {error ? <p className="map-status map-error">{error}</p> : null}
    </div>
  );
}
