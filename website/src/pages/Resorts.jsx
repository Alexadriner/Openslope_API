import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { apiFetch, fetchResortCount } from "../api/client";
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
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");
  const [hasMore, setHasMore] = useState(true);
  const [totalCount, setTotalCount] = useState(0);
  const [offset, setOffset] = useState(0);
  const sentinelRef = useRef(null);
  const offsetRef = useRef(0);
  const totalCountRef = useRef(0);
  const hasMoreRef = useRef(true);
  const loadingMoreRef = useRef(false);

  const PAGE_SIZE = 20;

  useEffect(() => {
    let cancelled = false;

    async function loadInitialResorts() {
      try {
        setLoading(true);
        setError("");
        const [data, countResponse] = await Promise.all([
          apiFetch(`/resorts?limit=${PAGE_SIZE}&offset=0`),
          fetchResortCount(),
        ]);

        if (!cancelled) {
          setResorts(Array.isArray(data) ? data : []);
          const initialCount = Array.isArray(data) ? data.length : 0;
          const total = Number(countResponse?.count ?? 0);
          setOffset(initialCount);
          offsetRef.current = initialCount;
          setHasMore(initialCount < total);
          hasMoreRef.current = initialCount < total;
          setTotalCount(total);
          totalCountRef.current = total;
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

    loadInitialResorts();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!sentinelRef.current || !hasMore || loading || loadingMore) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          loadMoreResorts();
        }
      },
      { rootMargin: "300px 0px" }
    );

    observer.observe(sentinelRef.current);

    return () => observer.disconnect();
  }, [hasMore, loading, loadingMore]);

  async function loadMoreResorts() {
    if (loadingMoreRef.current || !hasMoreRef.current) {
      return;
    }

    try {
      loadingMoreRef.current = true;
      setLoadingMore(true);
      const data = await apiFetch(`/resorts?limit=${PAGE_SIZE}&offset=${offsetRef.current}`);
      const nextResorts = Array.isArray(data) ? data : [];
      if (!nextResorts.length) {
        setHasMore(false);
        hasMoreRef.current = false;
        return;
      }

      setResorts((current) => [...current, ...nextResorts]);
      const nextOffset = offsetRef.current + nextResorts.length;
      setOffset(nextOffset);
      offsetRef.current = nextOffset;
      const shouldHaveMore = nextOffset < totalCountRef.current;
      setHasMore(shouldHaveMore);
      hasMoreRef.current = shouldHaveMore;
    } catch (err) {
      setError(err.message || "Failed to load more resorts");
    } finally {
      loadingMoreRef.current = false;
      setLoadingMore(false);
    }
  }

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
          <p>
            {query.trim()
              ? `${visibleResorts.length} resorts match your search.`
              : `Showing ${Math.min(resorts.length, totalCount)} of ${totalCount} resorts.`}
          </p>
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

      {hasMore ? (
        <div ref={sentinelRef} className="resorts-load-more" aria-hidden="true">
          {loadingMore ? "Loading more resorts…" : "Scroll to load more"}
        </div>
      ) : null}
    </div>
  );
}
