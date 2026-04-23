import { Link } from "react-router-dom";
import {
  formatStatusLabel,
  getActivities,
  getLocationLabel,
  getOpenMetric,
  getResortCoordinates,
  getResortStats,
  getSlopeDifficultyClass,
  getWebsites,
} from "../utils/resortData";

function formatTemperature(value) {
  return value == null ? "No data" : `${Number(value).toFixed(1)} °C`;
}

function formatSnow(value) {
  return value == null ? "No data" : `${value} cm`;
}

function formatDuration(value) {
  return value == null ? "No data" : `${value} s`;
}

function formatCapacity(value) {
  return value == null ? "No data" : `${value} p/h`;
}

export default function ResortContent({ resort }) {
  const stats = getResortStats(resort);
  const snapshot = resort.latest_snapshot ?? null;
  const activities = getActivities(resort);
  const coordinates = getResortCoordinates(resort);
  const websites = getWebsites(resort).slice(0, 3);

  return (
    <div className="page-container resort-page">
      <section className="resort-hero-card">
        <div className="resort-hero-copy">
          <p className="eyebrow">OpenSlope Resort Guide</p>
          <h1>{resort.name}</h1>
          <p className="resort-location">{getLocationLabel(resort)}</p>
          <p className="resort-summary">
            {formatStatusLabel(resort.status, "Status pending")} resort with {stats.slopeCount} slopes and{" "}
            {stats.liftCount} lifts in the database.
          </p>
          {activities.length > 0 && (
            <div className="chip-row">
              {activities.map((activity) => (
                <span key={activity} className="info-chip">
                  {activity}
                </span>
              ))}
            </div>
          )}
        </div>

        <div className="hero-metric-panel">
          <div className="hero-metric">
            <span className="metric-label">Lifts</span>
            <strong>{getOpenMetric(stats.openLiftCount, stats.totalLiftCount, stats.liftCount)}</strong>
          </div>
          <div className="hero-metric">
            <span className="metric-label">Slopes</span>
            <strong>{getOpenMetric(stats.openSlopeCount, stats.totalSlopeCount, stats.slopeCount)}</strong>
          </div>
          <div className="hero-metric">
            <span className="metric-label">Fresh snow</span>
            <strong>{formatSnow(snapshot?.new_snow_24h_cm)}</strong>
          </div>
          <div className="hero-metric">
            <span className="metric-label">Last update</span>
            <strong>{snapshot?.snapshot_time ? new Date(snapshot.snapshot_time).toLocaleString("de-DE") : "No live feed"}</strong>
          </div>
        </div>
      </section>

      <section className="resort-section resort-facts-grid">
        <article className="fact-card">
          <h2>Overview</h2>
          <p><strong>Type:</strong> {formatStatusLabel(resort.type, "Not specified")}</p>
          <p><strong>Resort status:</strong> {formatStatusLabel(resort.status)}</p>
          <p><strong>Coordinates:</strong> {coordinates ? `${coordinates[0].toFixed(4)}, ${coordinates[1].toFixed(4)}` : "No map point"}</p>
        </article>

        <article className="fact-card">
          <h2>Snow & Weather</h2>
          <p><strong>Valley snow:</strong> {formatSnow(snapshot?.snow_depth_valley_cm)}</p>
          <p><strong>Mountain snow:</strong> {formatSnow(snapshot?.snow_depth_mountain_cm)}</p>
          <p><strong>Valley temperature:</strong> {formatTemperature(snapshot?.temperature_valley_c)}</p>
          <p><strong>Mountain temperature:</strong> {formatTemperature(snapshot?.temperature_mountain_c)}</p>
        </article>

        <article className="fact-card">
          <h2>Links</h2>
          {websites.length > 0 ? (
            websites.map((entry) => (
              <p key={entry.url}>
                <a href={entry.url} target="_blank" rel="noreferrer">
                  {entry.label}
                </a>
              </p>
            ))
          ) : (
            <p>No official website stored yet.</p>
          )}
          <p>
            <Link to="/map">See all resorts on the map</Link>
          </p>
        </article>
      </section>

      <section className="tables-container">
        <article className="table-box">
          <h2>Slopes</h2>
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Difficulty</th>
                <th>Status</th>
                <th>Grooming</th>
              </tr>
            </thead>
            <tbody>
              {resort.slopes?.length ? (
                resort.slopes.map((slope) => (
                  <tr key={slope.id}>
                    <td>{slope.name ?? "Unnamed slope"}</td>
                    <td className={`difficulty ${getSlopeDifficultyClass(slope.difficulty)}`}>
                      {formatStatusLabel(slope.difficulty, "Unknown")}
                    </td>
                    <td>
                      <span className={`status-badge ${String(slope.status ?? "unknown").toLowerCase()}`}>
                        {formatStatusLabel(slope.status)}
                      </span>
                    </td>
                    <td>
                      <span className={`status-badge grooming ${String(slope.grooming ?? "unknown").toLowerCase()}`}>
                        {formatStatusLabel(slope.grooming)}
                      </span>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan="4">No slopes stored for this resort yet.</td>
                </tr>
              )}
            </tbody>
          </table>
        </article>

        <article className="table-box">
          <h2>Lifts</h2>
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Type</th>
                <th>Status</th>
                <th>Capacity</th>
                <th>Ride time</th>
              </tr>
            </thead>
            <tbody>
              {resort.lifts?.length ? (
                resort.lifts.map((lift) => (
                  <tr key={lift.id}>
                    <td>{lift.name ?? "Unnamed lift"}</td>
                    <td>{formatStatusLabel(lift.lift_type, "Unknown")}</td>
                    <td>
                      <span className={`status-badge ${String(lift.status ?? "unknown").toLowerCase()}`}>
                        {formatStatusLabel(lift.status)}
                      </span>
                    </td>
                    <td>{formatCapacity(lift.capacity)}</td>
                    <td>{formatDuration(lift.duration)}</td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan="5">No lifts stored for this resort yet.</td>
                </tr>
              )}
            </tbody>
          </table>
        </article>
      </section>
    </div>
  );
}
