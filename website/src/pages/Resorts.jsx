/**
 * Resorts List Component
 * 
 * This component displays a list of all ski resorts available in the OpenSlope system.
 * It provides a simple interface for users to browse and navigate to individual resort pages.
 * 
 * Features:
 * - Loads resort data from the API with caching support
 * - Displays resort names and countries
 * - Provides navigation links to individual resort detail pages
 * - Includes loading states and error handling
 * - Shows data source information
 * 
 * @author OpenSlope Team
 * @version 1.0.0
 */

import { useEffect, useState } from "react";
import { fetchResorts } from "../api/client";
import { Link } from "react-router-dom";
import "../stylesheets/base.css";

/**
 * Resorts component that renders a list of all available ski resorts
 * 
 * @returns {JSX.Element} - React component rendering the resorts list
 */
export default function Resorts() {
  // State management for resorts data, loading state, and errors
  const [resorts, setResorts] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  /**
   * Load resorts data from the API
   * Uses the 'names_only' transformation to optimize performance by fetching only
   * the necessary data for the list view (resort names and IDs)
   */
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

  // Loading state - displays while data is being fetched
  if (loading) {
    return (
      <div className="page-container">
        <h1>Resorts</h1>
        <p>Loading resorts...</p>
      </div>
    );
  }

  // Error state - displays when data loading fails
  if (error) {
    return (
      <div className="page-container">
        <h1>Resorts</h1>
        <p className="error-message">Error: {error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  // Success state - displays the list of resorts
  return (
    <div className="page-container">
      <h1>Resorts</h1>
      {/* Display the number of resorts and note about cached data */}
      <p className="data-source-note">Showing {resorts.length} resorts (cached data available)</p>
      
      {/* Render the list of resorts */}
      <ul>
        {resorts.map((resort) => (
          <li key={resort.id}>
            <Link to={`/resorts/${resort.id}`}>
              {resort.name} - {resort.country ?? "N/A"}
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}