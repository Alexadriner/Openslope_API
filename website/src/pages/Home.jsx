import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import SearchInputWithSuggestions from "../components/SearchInputWithSuggestions";
import { apiFetch, fetchResortCount, fetchSlopeCount, fetchLiftCount } from "../api/client";
import { getLocationLabel, getOpenMetric, getResortStats } from "../utils/resortData";

import "../stylesheets/base.css";
import "../stylesheets/home.css";

export default function Home() {
  const [stats, setStats] = useState({ resorts: 0, lifts: 0, slopes: 0 });
  const [featuredResorts, setFeaturedResorts] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function loadHomeData() {
      try {
        setLoading(true);

        // Load statistics (light requests)
        const [resortCountData, slopeCountData, liftCountData] = await Promise.all([
          fetchResortCount({ cacheTTL: 60_000 }),
          fetchSlopeCount({ cacheTTL: 60_000 }),
          fetchLiftCount({ cacheTTL: 60_000 })
        ]);

        if (!cancelled) {
          setStats({
            resorts: Number(resortCountData?.count ?? 0),
            lifts: Number(liftCountData?.count ?? 0),
            slopes: Number(slopeCountData?.count ?? 0)
          });

          // Load only top featured resorts (limit to 3 with pagination)
          const featuredData = await apiFetch("/resorts?limit=100&offset=0", {
            cacheTTL: 60_000
          });
          
          if (!cancelled && Array.isArray(featuredData)) {
            const featured = [...featuredData]
              .sort((left, right) => {
                const leftStats = getResortStats(left);
                const rightStats = getResortStats(right);
                return (rightStats.openSlopeCount ?? -1) - (leftStats.openSlopeCount ?? -1);
              })
              .slice(0, 3);
            setFeaturedResorts(featured);
          }
        }
      } catch (error) {
        console.error("Error loading home data:", error);
        if (!cancelled) {
          setStats({ resorts: 0, lifts: 0, slopes: 0 });
          setFeaturedResorts([]);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadHomeData();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="home-page">
      <section className="hero-shell">
        <div className="hero-copy">
          <p className="eyebrow">For ski days that need current info</p>
          <h1>Find resorts with live operations, slope inventory and mountain context.</h1>
          <p className="hero-text">
            OpenSlope turns raw resort data into a clear consumer view: where to go, what is open,
            and what each ski area actually offers.
          </p>
          <SearchInputWithSuggestions placeholder="Search for a resort or region" />
          <div className="hero-actions">
            <Link to="/resorts" className="home-button">Explore Resorts</Link>
            <Link to="/map" className="home-button secondary">Open Ski Map</Link>
          </div>
        </div>

        <div className="hero-stats-card">
          <h2>Database snapshot</h2>
          <div className="hero-stat-grid">
            <div>
              <span>Resorts</span>
              <strong>{stats.resorts}</strong>
            </div>
            <div>
              <span>Slopes</span>
              <strong>{stats.slopes}</strong>
            </div>
            <div>
              <span>Lifts</span>
              <strong>{stats.lifts}</strong>
            </div>
          </div>
          <p>
            Resorts with the richest live data rise to the top, so visitors quickly see where the
            mountain is actually moving.
          </p>
        </div>
      </section>

      {!loading && featuredResorts.length > 0 && (
        <section className="featured-section">
          <div className="section-heading">
            <p className="eyebrow">Featured right now</p>
            <h2>Resorts with the strongest live slope signal</h2>
          </div>

          <div className="featured-grid">
            {featuredResorts.map((resort) => {
              const stats = getResortStats(resort);
              return (
                <Link key={resort.id} to={`/resorts/${resort.id}`} className="featured-card">
                  <p className="featured-location">{getLocationLabel(resort)}</p>
                  <h3>{resort.name}</h3>
                  <p>{getOpenMetric(stats.openSlopeCount, stats.totalSlopeCount, stats.slopeCount)}</p>
                  <p>{getOpenMetric(stats.openLiftCount, stats.totalLiftCount, stats.liftCount)}</p>
                </Link>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
}
