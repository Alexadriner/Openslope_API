import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { apiFetch } from "../api/client";
import { getLocationLabel } from "../utils/resortData";
import "./stylesheets/searchInput.css";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";

export default function SearchInputWithSuggestions({ placeholder }) {
  const [query, setQuery] = useState("");
  const [allResorts, setAllResorts] = useState([]);
  const [hasLoadedResorts, setHasLoadedResorts] = useState(false);
  const navigate = useNavigate();

  // Only load resorts when user starts typing or when suggestions are about to be shown
  useEffect(() => {
    if (!query.trim() || hasLoadedResorts) {
      return;
    }

    let cancelled = false;

    async function loadResorts() {
      try {
        // Use paginated API to avoid loading all resorts at once
        // Load first batch of resorts for suggestions
        const data = await apiFetch("/resorts?limit=500&offset=0");
        if (!cancelled) {
          setAllResorts(Array.isArray(data) ? data : []);
          setHasLoadedResorts(true);
        }
      } catch (err) {
        console.error("Error loading resorts for search:", err);
      }
    }

    loadResorts();

    return () => {
      cancelled = true;
    };
  }, [query, hasLoadedResorts]);

  const suggestions = useMemo(() => {
    if (!query.trim()) {
      return [];
    }

    const normalizedQuery = query.toLowerCase();
    return allResorts
      .filter((resort) => {
        const location = getLocationLabel(resort, "").toLowerCase();
        return (
          resort.name.toLowerCase().includes(normalizedQuery) ||
          location.includes(normalizedQuery)
        );
      })
      .slice(0, 8);
  }, [query, allResorts]);

  function handleSelect(resort) {
    setQuery("");
    navigate(`/resorts/${resort.id}`);
  }

  function handleSubmit(event) {
    event.preventDefault();
    if (suggestions[0]) {
      handleSelect(suggestions[0]);
    }
  }

  return (
    <div className="search-wrapper">
      <form onSubmit={handleSubmit}>
        <input
          className="search-input"
          placeholder={placeholder}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <button className="search-button" aria-label="Search">
          <FontAwesomeIcon icon={faMagnifyingGlass} />
        </button>
      </form>

      {suggestions.length > 0 && (
        <ul className="suggestions-list">
          {suggestions.map((resort) => (
            <li
              key={resort.id}
              className="suggestion-item"
              onClick={() => handleSelect(resort)}
            >
              <strong>{resort.name}</strong>
              <span>{getLocationLabel(resort)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
