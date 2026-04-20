import { useEffect, useRef, useState } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import "leaflet.markercluster";
import "leaflet.markercluster/dist/MarkerCluster.css";
import "leaflet.markercluster/dist/MarkerCluster.Default.css";
import { fetchResortsForMap } from "../api/client";
import { formatStatusLabel, getLocationLabel, getResortStats } from "../utils/resortData";
import "../stylesheets/base.css";
import "../stylesheets/map.css";

const DEFAULT_CENTER = [46.8, 8.2];
const DEFAULT_ZOOM = 5;
const LIFTS_MIN_ZOOM = 10;
const SLOPES_MIN_ZOOM = 11;

const RESORT_MARKER_ICON = L.divIcon({
  className: "single-resort-marker-icon",
  html: '<div class="single-resort-marker-dot" aria-hidden="true"></div>',
  iconSize: [30, 30],
  iconAnchor: [15, 15],
  popupAnchor: [0, -15],
});

function getSlopeColor(difficulty) {
  const key = String(difficulty ?? "").toLowerCase().trim();
  if (key === "green") return "#30a46c";
  if (key === "blue") return "#2075c7";
  if (key === "red") return "#c43d36";
  if (key === "black") return "#1f2933";
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

function prepareResort(resort) {
  const coordinates = resort.coordinates;
  if (!coordinates) {
    return null;
  }

  return {
    marker: L.marker([coordinates.latitude, coordinates.longitude], {
      icon: RESORT_MARKER_ICON,
    }).bindPopup(createResortPopupHtml(resort)),
    point: L.latLng(coordinates.latitude, coordinates.longitude),
    lifts: (resort.lifts ?? [])
      .map((lift) =>
        createFeatureLayer(
          lift.geometry,
          {
            color: "#6b7280",
            weight: 2,
            opacity: 0.85,
          },
          `${lift.name ?? "Unnamed lift"}<br/>${formatStatusLabel(lift.status)}`
        )
      )
      .filter(Boolean),
    slopes: (resort.slopes ?? [])
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
      .filter(Boolean),
  };
}

export default function Map() {
  const containerRef = useRef(null);
  const mapRef = useRef(null);
  const clusterRef = useRef(null);
  const liftsLayerRef = useRef(null);
  const slopesLayerRef = useRef(null);
  const preparedResortsRef = useRef([]);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

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
      preparedResortsRef.current = [];
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    async function loadMapData() {
      if (!mapRef.current || !clusterRef.current || !liftsLayerRef.current || !slopesLayerRef.current) {
        return;
      }

      try {
        setLoading(true);
        setError("");

        const resorts = await fetchResortsForMap();
        if (cancelled) {
          return;
        }

        preparedResortsRef.current = resorts.map(prepareResort).filter(Boolean);

        clusterRef.current.clearLayers();
        preparedResortsRef.current.forEach((resort) => {
          clusterRef.current.addLayer(resort.marker);
        });

        const bounds = L.latLngBounds(preparedResortsRef.current.map((resort) => resort.point));
        if (bounds.isValid()) {
          mapRef.current.fitBounds(bounds.pad(0.18));
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

    loadMapData();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const map = mapRef.current;
    const liftsLayer = liftsLayerRef.current;
    const slopesLayer = slopesLayerRef.current;

    if (!map || !liftsLayer || !slopesLayer) {
      return undefined;
    }

    const refreshVisibleLayers = () => {
      const visibleBounds = map.getBounds().pad(0.12);
      const zoom = map.getZoom();
      const showLifts = zoom >= LIFTS_MIN_ZOOM;
      const showSlopes = zoom >= SLOPES_MIN_ZOOM;

      liftsLayer.clearLayers();
      slopesLayer.clearLayers();

      preparedResortsRef.current.forEach((resort) => {
        if (showLifts) {
          resort.lifts.forEach((lift) => {
            if (visibleBounds.intersects(lift.bounds)) {
              liftsLayer.addLayer(lift.layer);
            }
          });
        }

        if (showSlopes) {
          resort.slopes.forEach((slope) => {
            if (visibleBounds.intersects(slope.bounds)) {
              slopesLayer.addLayer(slope.layer);
            }
          });
        }
      });
    };

    refreshVisibleLayers();
    map.on("moveend", refreshVisibleLayers);
    map.on("zoomend", refreshVisibleLayers);

    return () => {
      map.off("moveend", refreshVisibleLayers);
      map.off("zoomend", refreshVisibleLayers);
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
