import "../stylesheets/base.css";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

export default function UserDashboard() {
  const { logout } = useAuth();
  const navigate = useNavigate();

  function handleLogout() {
    logout();
    navigate("/login");
  }

  return (
    <div className="page-container">
      <h1>Benutzer-Dashboard</h1>

      {/* Profil */}
      <section style={{ marginBottom: "2rem" }}>
        <h2>Profil</h2>

        <p>
          <strong>Name:</strong> Gespeichert
          <br />
          <strong>E-Mail:</strong> Gespeichert
          <br />
          <strong>Abonnement:</strong> Free
        </p>

        <p style={{ fontSize: "0.9rem", color: "#666" }}>
          (Profildaten werden später aus dem Backend geladen)
        </p>
      </section>

      {/* API Keys */}
      <section style={{ marginBottom: "2rem" }}>
        <h2>API-Keys</h2>

        <p>
          Dein API-Key wurde bei der Registrierung erstellt und sicher
          gespeichert.
        </p>

        <p style={{ color: "#555" }}>
          Aus Sicherheitsgründen wird er nicht mehr angezeigt.
        </p>

        <ul>
          <li>
            <code>••••••••••••••••</code> – aktiv – Rate Limit: 1000 req/Tag
          </li>
        </ul>

        <button disabled>
          Neuen API-Key erstellen (coming soon)
        </button>
      </section>

      {/* Account */}
      <section>
        <h2>Account-Einstellungen</h2>

        <div style={{ display: "flex", gap: "1rem", flexWrap: "wrap" }}>
          <button disabled>Passwort ändern</button>
          <button disabled>E-Mail ändern</button>

          <button
            onClick={handleLogout}
            style={{ color: "red", borderColor: "red" }}
          >
            Logout
          </button>
        </div>
      </section>
    </div>
  );
}
