import "../stylesheets/base.css";
import "../stylesheets/home.css";

import "../components/stylesheets/searchInput.css"
import SearchInput from "../components/SearchInput";
export default function Home() {
  return (
    <div className="page-container">
      <h1>SkiAPI & Skigebiete</h1>
      <p>Finde Skigebiete oder nutze unsere leistungsstarke API.</p>

      <div>
        <SearchInput placeholder="Skigebiet suchen..." />
        <button className="home-button">API ausprobieren</button>
        <button className="home-button">Skigebiete entdecken</button>
      </div>
    </div>
  );
}