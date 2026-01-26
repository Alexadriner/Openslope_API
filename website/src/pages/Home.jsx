import SearchInputWithSuggestions from "../components/SearchInputWithSuggestions";
import { Link } from "react-router-dom";

import "../stylesheets/base.css";
import "../stylesheets/home.css"

export default function Home() {
  return (
    <div className="page-container">
      <h1>SkiAPI & Skigebiete</h1>
      <p>Finde Skigebiete oder nutze unsere leistungsstarke API.</p>

      <div>
        <SearchInputWithSuggestions placeholder="Skigebiet suchen..." />
        <Link to="/api/demo" className="home-button">API ausprobieren</Link>
        <Link to="/resorts" className="home-button">Skigebiete entdecken</Link>
      </div>
    </div>
  );
}
