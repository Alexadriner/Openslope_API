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
  const navigate = useNavigate();

  useEffect(() => {
    let cancelled = false;

    apiFetch("/resorts?summary=true")
      .then((data) => {
        if (!cancelled) {
          setAllResorts(Array.isArray(data) ? data : []);
        }
      })
      .catch((err) => console.error(err));

    return () => {
      cancelled = true;
    };
  }, []);

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
