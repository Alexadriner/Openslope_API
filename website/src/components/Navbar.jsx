import { Link } from "react-router-dom";
import "./stylesheets/navbar.css";


export default function Navbar() {
return (
<nav className="navbar">
{/* Website-Funktionen */}
<div className="nav-section">
<Link className="nav-button" to="/">Start</Link>
<Link className="nav-button" to="/resorts">Skigebiete</Link>
<Link className="nav-button" to="/map">Skimap</Link>
<Link className="nav-button" to="/contact">Kontakt</Link>
</div>


{/* Account / API */}
<div className="nav-section">
<Link className="nav-button" to="/api">API</Link>
<Link className="nav-button" to="/api/demo">Demo</Link>
<Link className="nav-button" to="/user">Account</Link>
<Link className="nav-button" to="/login">Login</Link>
</div>
</nav>
);
}