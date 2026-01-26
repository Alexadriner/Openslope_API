import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { apiFetch } from "../api/client";
import "../stylesheets/base.css";

export default function ResortPage() {
  const { name } = useParams();
  const [resort, setResort] = useState(null);

  useEffect(() => {
    apiFetch("resorts")
      .then((data) => {
        const found = data.find((r) => r.name === name);
        setResort(found);
      })
      .catch((err) => console.error(err));
  }, [name]);

  if (!resort) return <p>Lade Resort...</p>;

  return (
    <div className="page-container">
      <h1>{resort.name}</h1>
      <p>{resort.country} {resort.region ? `- ${resort.region}` : ""}</p>
      <p>Kontinent: {resort.continent}</p>
      <p>Ortshöhe: {resort.village_altitude_m ?? "N/A"} m</p>
      <p>Min. Höhe: {resort.min_altitude_m ?? "N/A"} m</p>
      <p>Max. Höhe: {resort.max_altitude_m ?? "N/A"} m</p>
      <p>Ski Area: {resort.ski_area_name ?? "N/A"} ({resort.ski_area_type ?? "N/A"})</p>
      {resort.latitude && resort.longitude && (
        <p>Koordinaten: {resort.latitude}, {resort.longitude}</p>
      )}
    </div>
  );
}