import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import SearchInputWithSuggestions from "../components/SearchInputWithSuggestions";
import { fetchResorts } from "../api/client";
import { getLocationLabel, getOpenMetric, getResortStats } from "../utils/resortData";

import "../stylesheets/base.css";
import "../stylesheets/home.css";

export default function Home() {
  const [resorts, setResorts] = useState([]);

  useEffect(() => {
    let cancelled = false;

    fetchResorts(null, { cacheTTL: 60_000 })
      .then((data) => {
        if (!cancelled) {
          setResorts(Array.isArray(data) ? data : []);
        }
      })
      .catch((error) => console.error(error));

    return () => {
      cancelled = true;
    };
  }, []);

  const featuredResorts = [...resorts]
    .sort((left, right) => {
      const leftStats = getResortStats(left);
      const rightStats = getResortStats(right);
      return (rightStats.openSlopeCount ?? -1) - (leftStats.openSlopeCount ?? -1);
    })
    .slice(0, 3);

  const totals = resorts.reduce(
    (accumulator, resort) => {
      const stats = getResortStats(resort);
      accumulator.resorts += 1;
      accumulator.lifts += stats.liftCount;
      accumulator.slopes += stats.slopeCount;
      return accumulator;
    },
    { resorts: 0, lifts: 0, slopes: 0 }
  );

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
              <strong>{totals.resorts}</strong>
            </div>
            <div>
              <span>Slopes</span>
              <strong>{totals.slopes}</strong>
            </div>
            <div>
              <span>Lifts</span>
              <strong>{totals.lifts}</strong>
            </div>
          </div>
          <p>
            Resorts with the richest live data rise to the top, so visitors quickly see where the
            mountain is actually moving.
          </p>
        </div>
      </section>

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
    </div>
  );
}
