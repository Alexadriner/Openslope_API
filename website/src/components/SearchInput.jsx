import "../stylesheets/base.css";
import "./stylesheets/searchInput.css";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";


export default function SearchInput({ placeholder }) {
  return (
    <div className="search-wrapper">
      <input
        className="search-input"
        placeholder={placeholder}
      />
      <button className="search-button" aria-label="Suchen">
        <FontAwesomeIcon icon={faMagnifyingGlass} />
    </button>
    </div>
  );
}