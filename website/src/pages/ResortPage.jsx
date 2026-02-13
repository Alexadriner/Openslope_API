import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { apiFetch } from "../api/client";

import "../stylesheets/resort-page.css";
import "../stylesheets/base.css";

export default function ResortPage() {
  const { name } = useParams();

  const [resort, setResort] = useState(null);
  const [slopes, setSlopes] = useState([]);
  const [lifts, setLifts] = useState([]);

  useEffect(() => {
    async function loadData() {
      try {
        const resortsData = await apiFetch("resorts");
        const slopesData = await apiFetch("slopes");
        const liftsData = await apiFetch("lifts");

        const foundResort = resortsData.find((r) => r.name === name);
        if (!foundResort) return;

        const resortSlopes = slopesData.filter((s) => s.resort_id === foundResort.id);
        const resortLifts = liftsData.filter((l) => l.resort_id === foundResort.id);

        setResort(foundResort);
        setSlopes(resortSlopes);
        setLifts(resortLifts);
      } catch (err) {
        console.error("API error:", err);
      }
    }

    loadData();
  }, [name]);

  if (!resort) return <p>Loading resort...</p>;

  return (
    <div className="page-container resort-page">
      <div className="resort-header">
        <h1>{resort.name}</h1>

        <p>
          {resort.country}
          {resort.region ? ` - ${resort.region}` : ""}
        </p>

        <div className="resort-info-grid">
          <p>Continent: {resort.continent ?? "N/A"}</p>
          <p>Village altitude: {resort.village_altitude_m ?? "N/A"} m</p>
          <p>Min: {resort.min_altitude_m ?? "N/A"} m</p>
          <p>Max: {resort.max_altitude_m ?? "N/A"} m</p>
          <p>Ski area: {resort.ski_area_name ?? "N/A"}</p>

          {resort.latitude && resort.longitude && (
            <p>
              Coordinates: {resort.latitude}, {resort.longitude}
            </p>
          )}
        </div>
      </div>

      <div className="tables-container">
        <div className="table-box">
          <h2>Slopes</h2>

          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Difficulty</th>
                <th></th>
              </tr>
            </thead>

            <tbody>
              {slopes.map((slope) => (
                <tr key={slope.id}>
                  <td>{slope.name ?? "Unknown"}</td>

                  <td className={`difficulty ${slope.difficulty}`}>
                    {slope.difficulty}
                  </td>

                  <td>
                    <span className={`difficulty-dot ${slope.difficulty}`} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="table-box">
          <h2>Lifts</h2>

          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Type</th>
              </tr>
            </thead>

            <tbody>
              {lifts.map((lift) => (
                <tr key={lift.id}>
                  <td>{lift.name ?? "Unnamed"}</td>
                  <td>{lift.lift_type}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
