import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { fetchResorts } from "../api/client";
import {
  formatStatusLabel,
  getActivities,
  getLocationLabel,
  getOpenMetric,
  getResortStats,
} from "../utils/resortData";
import "../stylesheets/base.css";
import "../stylesheets/resorts.css";

export default function Resorts() {
  const [resorts, setResorts] = useState([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function loadResorts() {
      try {
        setLoading(true);
        setError("");
        const data = await fetchResorts();
        if (!cancelled) {
          setResorts(Array.isArray(data) ? data : []);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err.message || "Failed to load resorts");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadResorts();

    return () => {
      cancelled = true;
    };
  }, []);

  const visibleResorts = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    const filtered = normalizedQuery
      ? resorts.filter((resort) => {
          const haystack = `${resort.name} ${getLocationLabel(resort)} ${getActivities(resort).join(" ")}`.toLowerCase();
          return haystack.includes(normalizedQuery);
        })
      : resorts;

    return [...filtered].sort((left, right) => {
      const leftStats = getResortStats(left);
      const rightStats = getResortStats(right);
      return (rightStats.openSlopeCount ?? -1) - (leftStats.openSlopeCount ?? -1);
    });
  }, [query, resorts]);

  if (loading) {
    return <div className="resorts-page"><p>Loading resorts...</p></div>;
  }

  if (error) {
    return <div className="resorts-page"><p className="error-message">{error}</p></div>;
  }

  return (
    <div className="resorts-page">
      <section className="resorts-header">
        <div>
          <p className="eyebrow">Resort discovery</p>
          <h1>Ski resorts ready for a consumer-friendly overview</h1>
          <p>{visibleResorts.length} resorts match your search.</p>
        </div>

        <label className="resorts-filter">
          <span className="sr-only">Filter resorts</span>
          <input
            type="search"
            placeholder="Filter by resort, place or activity"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>
      </section>

      <section className="resorts-grid">
        {visibleResorts.map((resort) => {
          const stats = getResortStats(resort);
          const activities = getActivities(resort).slice(0, 3);

          return (
            <Link key={resort.id} to={`/resorts/${resort.id}`} className="resort-card-link">
              <article className="resort-card">
                <p className="resort-card-location">{getLocationLabel(resort)}</p>
                <h2>{resort.name}</h2>
                <p>{activities.length ? activities.join(" · ") : "Ski information available in database"}</p>

                <div className="resort-card-metrics">
                  <div>
                    <span>Lifts</span>
                    <strong>{getOpenMetric(stats.openLiftCount, stats.totalLiftCount, stats.liftCount)}</strong>
                  </div>
                  <div>
                    <span>Slopes</span>
                    <strong>{getOpenMetric(stats.openSlopeCount, stats.totalSlopeCount, stats.slopeCount)}</strong>
                  </div>
                </div>

                <div className="resort-card-status">
                  <span className={`status-badge ${String(resort.status ?? "unknown").toLowerCase()}`}>
                    {formatStatusLabel(resort.status)}
                  </span>
                  <span>{resort.latest_snapshot?.new_snow_24h_cm != null ? `+${resort.latest_snapshot.new_snow_24h_cm} cm new snow` : "No new snow data"}</span>
                </div>
              </article>
            </Link>
          );
        })}
      </section>
    </div>
  );
}
