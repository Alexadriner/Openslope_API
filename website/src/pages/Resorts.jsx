import { useEffect, useState } from "react";
import { fetchResorts } from "../api/client";
import { Link } from "react-router-dom";
import "../stylesheets/base.css";

export default function Resorts() {
  const [resorts, setResorts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const loadResorts = async () => {
      try {
        setLoading(true);
        // Use names_only transformation since we only need names and IDs for the list
        const data = await fetchResorts("names_only");
        setResorts(data);
      } catch (err) {
        setError(err.message || "Failed to load resorts");
      } finally {
        setLoading(false);
      }
    };

    loadResorts();
  }, []);

  if (loading) {
    return (
      <div className="page-container">
        <h1>Resorts</h1>
        <p>Loading resorts...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="page-container">
        <h1>Resorts</h1>
        <p className="error-message">Error: {error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  return (
    <div className="page-container">
      <h1>Resorts</h1>
      <p className="data-source-note">Showing {resorts.length} resorts (cached data available)</p>
      <ul>
        {resorts.map((r) => (
          <li key={r.id}>
            <Link to={`/resorts/${r.id}`}>
              {r.name} - {r.country ?? "N/A"}
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}
