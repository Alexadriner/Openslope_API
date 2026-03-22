/**
 * Interactive Ski Map Component
 * 
 * This component renders an interactive map using Leaflet.js to display ski resorts,
 * slopes, and lifts worldwide. It provides a comprehensive visualization of ski area
 * infrastructure with clustering for better performance and user experience.
 * 
 * Features:
 * - Interactive map with OpenStreetMap tiles
 * - Resort clustering for better performance
 * - Slope and lift visualization with color coding
 * - Responsive design with zoom-based layer visibility
 * - Popup information for resorts
 * 
 * @author OpenSlope Team
 * @version 1.0.0
 */

import { useEffect, useRef, useState } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "leaflet.markercluster";
import "leaflet.markercluster/dist/MarkerCluster.css";
import "leaflet.markercluster/dist/MarkerCluster.Default.css";
import { fetchResortsForMap, fetchSlopesForMap, fetchLiftsForMap } from "../api/client";
import "../stylesheets/base.css";
import "../stylesheets/map.css";

/**
 * Custom marker icon for individual resorts
 * Uses a div icon with CSS styling for better customization
 */
const RESORT_MARKER_ICON = L.divIcon({
  className: "single-resort-marker-icon",
  html: '<div class="single-resort-marker-dot" aria-hidden="true"></div>',
  iconSize: [30, 30],
  iconAnchor: [15, 15],
  popupAnchor: [0, -15],
});

/**
 * Default map center and zoom level
 * Centered on the Alps region for optimal ski resort visibility
 */
const DEFAULT_CENTER = [46.8, 8.2];
const DEFAULT_ZOOM = 5;

/**
 * Zoom levels for different layer types
 * Lifts and slopes are only shown at higher zoom levels for performance
 */
const LIFTS_MIN_ZOOM = 9;
const SLOPES_MIN_ZOOM = 10;

/**
 * Clustering radius in pixels
 * Resorts within this distance will be clustered together
 */
const CLUSTER_PIXEL_RADIUS = 55;

/**
 * Convert a value to a number or null
 * @param {*} value - Value to convert
 * @returns {number|null} - Parsed number or null
 */
function toNumberOrNull(value) {
  if (value == null) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

/**
 * Extract resort coordinates from various data formats
 * Handles both new geography structure and legacy latitude/longitude fields
 * 
 * @param {Object} resort - Resort data object
 * @returns {Array|null} - [latitude, longitude] array or null
 */
function getResortCoordinates(resort) {
  const latitude = toNumberOrNull(resort.geography?.coordinates?.latitude ?? resort.latitude);
  const longitude = toNumberOrNull(resort.geography?.coordinates?.longitude ?? resort.longitude);

  if (latitude == null || longitude == null) return null;

  return [latitude, longitude];
}

/**
 * Get color for slope based on difficulty level
 * 
 * @param {string} difficulty - Slope difficulty (green, blue, red, black)
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
 * Create a line layer between two points
 * Used for drawing lifts and slopes as simple lines
 * 
 * @param {number} startLat - Starting latitude
 * @param {number} startLon - Starting longitude
 * @param {number} endLat - Ending latitude
 * @param {number} endLon - Ending longitude
 * @param {Object} style - Leaflet line style options
 * @returns {Object|null} - Layer and bounds object or null
 */
function createLineEntry(startLat, startLon, endLat, endLon, style) {
  const aLat = toNumberOrNull(startLat);
  const aLon = toNumberOrNull(startLon);
  const bLat = toNumberOrNull(endLat);
  const bLon = toNumberOrNull(endLon);

  if (aLat == null || aLon == null || bLat == null || bLon == null) {
    return null;
  }

  const layer = L.polyline(
    [
      [aLat, aLon],
      [bLat, bLon],
    ],
    style
  );

  return {
    layer,
    bounds: L.latLngBounds(
      [aLat, aLon],
      [bLat, bLon]
    ),
  };
}

/**
 * Create a GeoJSON layer from GeoJSON data
 * Used for slopes with complex geometry and direction information
 * 
 * @param {Object} geoJsonData - GeoJSON feature collection
 * @param {Object} style - Leaflet style options
 * @returns {Object|null} - Layer and bounds object or null
 */
function createGeoJsonEntry(geoJsonData, style) {
  if (!geoJsonData || !geoJsonData.features || geoJsonData.features.length === 0) {
    return null;
  }

  const layer = L.geoJSON(geoJsonData, {
    style: style,
    onEachFeature: function (feature, layer) {
      if (feature.properties && feature.properties.direction) {
        const direction = feature.properties.direction;
        const latlngs = layer.getLatLngs();
        if (latlngs && latlngs.length > 0) {
          const midPoint = latlngs[Math.floor(latlngs.length / 2)];
          L.marker(midPoint, {
            icon: L.divIcon({
              className: 'slope-direction-icon',
              html: `<div class="slope-direction-arrow" style="transform: rotate(${direction}deg)"></div>`,
              iconSize: [20, 20],
              iconAnchor: [10, 10]
            })
          }).addTo(layer);
          layer.bindPopup(`Direction: ${direction}°`);
        }
      }
    }
  });

  const bounds = layer.getBounds();
  return {
    layer,
    bounds: bounds.isValid() ? bounds : null
  };
}

/**
 * Create HTML content for resort popup
 * 
 * @param {Object} resort - Resort data object
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
 * Create a marker for a resort with popup
 * 
 * @param {Object} resort - Prepared resort object
 * @returns {L.Marker} - Leaflet marker instance
 */
function createResortMarker(resort) {
  return L.marker(resort.resortLatLng, { icon: RESORT_MARKER_ICON }).bindPopup(resort.popupHtml);
}

/**
 * Split visible resorts into clustered and single groups
 * Resorts that are close together will be clustered, others shown individually
 * 
 * @param {Array} visibleResorts - Array of visible resort objects
 * @param {L.Map} map - Leaflet map instance
 * @returns {Object} - Object with singles and grouped arrays
 */
function splitVisibleResortsByProximity(visibleResorts, map) {
  const groupedIndexes = new Set();
  const zoom = map.getZoom();
  const projected = visibleResorts.map((resort) => map.project(resort.resortLatLng, zoom));

  for (let i = 0; i < projected.length; i += 1) {
    for (let j = i + 1; j < projected.length; j += 1) {
      if (projected[i].distanceTo(projected[j]) <= CLUSTER_PIXEL_RADIUS) {
        groupedIndexes.add(i);
        groupedIndexes.add(j);
      }
    }
  }

  const singles = [];
  const grouped = [];

  for (let i = 0; i < visibleResorts.length; i += 1) {
    if (groupedIndexes.has(i)) {
      grouped.push(visibleResorts[i]);
    } else {
      singles.push(visibleResorts[i]);
    }
  }

  return { singles, grouped };
}

/**
 * Prepare resort data for map rendering
 * Extracts coordinates, creates popup HTML, and processes lifts and slopes
 * 
 * @param {Object} resort - Raw resort data
 * @returns {Object} - Prepared resort object with layers and metadata
 */
function prepareResort(resort) {
  const coordinates = getResortCoordinates(resort);
  const resortLatLng = coordinates ? L.latLng(coordinates[0], coordinates[1]) : null;
  const popupHtml = createResortPopupHtml(resort);

  const lifts = (resort.lifts ?? [])
    .map((lift) =>
      createLineEntry(
        lift.geometry?.start?.latitude ?? lift.lat_start,
        lift.geometry?.start?.longitude ?? lift.lon_start,
        lift.geometry?.end?.latitude ?? lift.lat_end,
        lift.geometry?.end?.longitude ?? lift.lon_end,
        {
          color: "#8e8e8e",
          weight: 2,
          opacity: 0.8,
        }
      )
    )
    .filter(Boolean);

  const slopes = (resort.slopes ?? [])
    .map((slope) => {
      // Check if GeoJSON data is available
      if (slope.geometry?.path && slope.geometry.path.length > 0) {
        // Create GeoJSON data structure
        const geoJsonData = {
          type: "FeatureCollection",
          features: [{
            type: "Feature",
            properties: {
              direction: slope.geometry.direction
            },
            geometry: {
              type: "LineString",
              coordinates: slope.geometry.path.map(point => [point.longitude, point.latitude])
            }
          }]
        };

        return createGeoJsonEntry(geoJsonData, {
          color: getSlopeColor(slope.difficulty ?? slope.display?.difficulty),
          weight: 2.2,
          opacity: 0.95,
        });
      } else {
        // Fallback to line entry if no GeoJSON data
        return createLineEntry(
          slope.geometry?.start?.latitude ?? slope.lat_start,
          slope.geometry?.start?.longitude ?? slope.lon_start,
          slope.geometry?.end?.latitude ?? slope.lat_end,
          slope.geometry?.end?.longitude ?? slope.lon_end,
          {
            color: getSlopeColor(slope.difficulty ?? slope.display?.difficulty),
            weight: 2.2,
            opacity: 0.95,
          }
        );
      }
    })
    .filter(Boolean);

  return {
    resortLatLng,
    popupHtml,
    lifts,
    slopes,
  };
}

/**
 * Main Map component
 * 
 * Renders an interactive ski map with resorts, slopes, and lifts.
 * Uses Leaflet.js for map rendering and clustering for performance optimization.
 * 
 * @returns {JSX.Element} - React component
 */
export default function Map() {
  // Refs for map elements and data
  const containerRef = useRef(null);
  const mapRef = useRef(null);
  const clusterMarkersRef = useRef(null);
  const singleMarkersRef = useRef(null);
  const liftsLayerRef = useRef(null);
  const slopesLayerRef = useRef(null);
  const preparedResortsRef = useRef([]);

  // State for loading and error handling
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Initialize map on component mount
  useEffect(() => {
    if (mapRef.current || !containerRef.current) return;

    const map = L.map(containerRef.current, {
      center: DEFAULT_CENTER,
      zoom: DEFAULT_ZOOM,
      minZoom: 2,
      worldCopyJump: true,
    });

    // Add OpenStreetMap tile layer
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      maxZoom: 19,
    }).addTo(map);

    // Initialize layer groups
    const clusterMarkers = L.markerClusterGroup({
      showCoverageOnHover: false,
      spiderfyOnMaxZoom: true,
      maxClusterRadius: CLUSTER_PIXEL_RADIUS,
    });

    const singleMarkers = L.layerGroup();
    const liftsLayer = L.layerGroup();
    const slopesLayer = L.layerGroup();

    clusterMarkers.addTo(map);
    singleMarkers.addTo(map);
    liftsLayer.addTo(map);
    slopesLayer.addTo(map);

    // Store references
    mapRef.current = map;
    clusterMarkersRef.current = clusterMarkers;
    singleMarkersRef.current = singleMarkers;
    liftsLayerRef.current = liftsLayer;
    slopesLayerRef.current = slopesLayer;

    // Cleanup on unmount
    return () => {
      map.off("moveend");
      map.off("zoomend");
      map.remove();
      mapRef.current = null;
      clusterMarkersRef.current = null;
      singleMarkersRef.current = null;
      liftsLayerRef.current = null;
      slopesLayerRef.current = null;
      preparedResortsRef.current = [];
    };
  }, []);

  // Load map data on component mount
  useEffect(() => {
    async function loadMapData() {
      if (
        !mapRef.current ||
        !clusterMarkersRef.current ||
        !singleMarkersRef.current ||
        !liftsLayerRef.current ||
        !slopesLayerRef.current
      ) {
        return;
      }

      setLoading(true);
      setError("");

      try {
        // Load resorts optimized for map rendering
        const resorts = await fetchResortsForMap();
        preparedResortsRef.current = resorts.map(prepareResort);

        // Calculate bounds and fit map to show all resorts
        const bounds = L.latLngBounds([]);
        for (const resort of preparedResortsRef.current) {
          if (resort.resortLatLng) bounds.extend(resort.resortLatLng);
        }

        if (bounds.isValid()) {
          mapRef.current.fitBounds(bounds.pad(0.15));
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Map data could not be loaded.");
      } finally {
        setLoading(false);
      }
    }

    loadMapData();
  }, []);

  // Handle map interactions and layer visibility
  useEffect(() => {
    const map = mapRef.current;
    const clusterMarkers = clusterMarkersRef.current;
    const singleMarkers = singleMarkersRef.current;
    const liftsLayer = liftsLayerRef.current;
    const slopesLayer = slopesLayerRef.current;
    if (!map || !clusterMarkers || !singleMarkers || !liftsLayer || !slopesLayer) return;

    /**
     * Refresh visible layers based on current map view
     * Shows/hides layers based on zoom level and viewport
     */
    const refreshVisibleLayers = () => {
      const visibleBounds = map.getBounds().pad(0.1);
      const zoom = map.getZoom();
      const showLifts = zoom >= LIFTS_MIN_ZOOM;
      const showSlopes = zoom >= SLOPES_MIN_ZOOM;

      // Clear all layers
      clusterMarkers.clearLayers();
      singleMarkers.clearLayers();
      liftsLayer.clearLayers();
      slopesLayer.clearLayers();

      // Get visible resorts and split into clusters and singles
      const visibleResorts = preparedResortsRef.current.filter(
        (resort) => resort.resortLatLng && visibleBounds.contains(resort.resortLatLng)
      );
      const { singles, grouped } = splitVisibleResortsByProximity(visibleResorts, map);

      // Add resort markers
      for (const resort of grouped) {
        clusterMarkers.addLayer(createResortMarker(resort));
      }

      for (const resort of singles) {
        singleMarkers.addLayer(createResortMarker(resort));
      }

      // Add lifts and slopes based on zoom level
      for (const resort of preparedResortsRef.current) {
        if (showLifts) {
          for (const lift of resort.lifts) {
            if (visibleBounds.intersects(lift.bounds)) {
              liftsLayer.addLayer(lift.layer);
            }
          }
        }

        if (showSlopes) {
          for (const slope of resort.slopes) {
            if (visibleBounds.intersects(slope.bounds)) {
              slopesLayer.addLayer(slope.layer);
            }
          }
        }
      }
    };

    // Initial refresh and event listeners
    refreshVisibleLayers();
    map.on("moveend", refreshVisibleLayers);
    map.on("zoomend", refreshVisibleLayers);

    // Cleanup event listeners
    return () => {
      map.off("moveend", refreshVisibleLayers);
      map.off("zoomend", refreshVisibleLayers);
    };
  }, [loading]);

  // Render the map component
  return (
    <div className="page-container map-page">
      <h1>Ski Map</h1>
      <p className="map-subtitle">
        Interactive skimap to find resorts around the world!
      </p>

      <div className="map-frame">
        <div ref={containerRef} className="ski-map-canvas" />
      </div>

      {loading && <p className="map-status">Lade Kartendaten...</p>}
      {error && <p className="map-status map-error">Fehler: {error}</p>}
    </div>
  );
}