import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { apiFetch } from "../api/client";
import "../stylesheets/resort-page.css";
import "../stylesheets/base.css";

export default function ResortDetail() {
  const { id } = useParams();
  const [resort, setResort] = useState(null);

  useEffect(() => {
    apiFetch(`/resorts/${id}`).then(setResort);
  }, [id]);

  if (!resort) return <p>Loading...</p>;

  const country = resort.geography?.country ?? resort.country ?? "N/A";
  const region = resort.geography?.region ?? resort.region;
  const continent = resort.geography?.continent ?? resort.continent;

  const latitude = resort.geography?.coordinates?.latitude ?? resort.latitude;
  const longitude = resort.geography?.coordinates?.longitude ?? resort.longitude;

  const villageAltitude = resort.altitude?.village_m ?? resort.village_altitude_m;
  const minAltitude = resort.altitude?.min_m ?? resort.min_altitude_m;
  const maxAltitude = resort.altitude?.max_m ?? resort.max_altitude_m;

  const skiAreaName = resort.ski_area?.name ?? resort.ski_area_name;
  const skiAreaType = resort.ski_area?.area_type ?? resort.ski_area_type;

  const lifts = resort.lifts ?? [];
  const slopes = resort.slopes ?? [];

  return (
    <div className="page-container resort-page">
      <div className="resort-header">
        <h1>{resort.name}</h1>

        <p>
          {country}
          {region ? ` - ${region}` : ""}
        </p>

        <div className="resort-info-grid">
          <p>Continent: {continent ?? "N/A"}</p>
          <p>Village altitude: {villageAltitude ?? "N/A"} m</p>
          <p>Min altitude: {minAltitude ?? "N/A"} m</p>
          <p>Max altitude: {maxAltitude ?? "N/A"} m</p>
          <p>Ski area: {skiAreaName ?? "N/A"}</p>
          <p>Ski area type: {skiAreaType ?? "N/A"}</p>
          <p>Total slopes: {slopes.length}</p>
          <p>Total lifts: {lifts.length}</p>
          <p>
            Coordinates:{" "}
            {latitude != null && longitude != null ? `${latitude}, ${longitude}` : "N/A"}
          </p>
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
              </tr>
            </thead>

            <tbody>
              {slopes.map((slope) => (
                <tr key={slope.id}>
                  <td>{slope.name ?? "Unknown"}</td>
                  <td className={`difficulty ${slope.difficulty}`}>
                    {slope.difficulty ?? "N/A"}
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
                  <td>{lift.lift_type ?? "N/A"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
