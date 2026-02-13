import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { apiFetch } from "../api/client";
import "../stylesheets/base.css";

export default function ResortDetail() {
  const { id } = useParams();
  const [resort, setResort] = useState(null);

  useEffect(() => {
    apiFetch(`/resorts/${id}`).then(setResort);
  }, [id]);

  if (!resort) return <p>Loading...</p>;

  return (
    <div className="page-container">
      <h1>{resort.name}</h1>
      <p>{resort.country} - {resort.region}</p>
      <p>Altitude: {resort.max_altitude_m ?? "N/A"} m</p>
    </div>
  );
}
