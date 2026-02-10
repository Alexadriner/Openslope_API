import { useState } from "react";
import { signup } from "../api/auth";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

export default function Signup() {
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [apiKey, setApiKey] = useState(null);

  const { saveKey } = useAuth();
  const navigate = useNavigate();

  async function handleSubmit(e) {
    e.preventDefault();

    try {
      const data = await signup(username, email, password);

      // speichern
      saveKey(data.api_key);

      // EINMAL anzeigen
      setApiKey(data.api_key);
    } catch (err) {
      alert(err.message);
    }
  }

  // Wenn Key da ist: Warnseite
  if (apiKey) {
    return (
      <div className="page-container">
        <h1>Wichtig!</h1>

        <p>Speichere deinen API-Key jetzt:</p>

        <code>{apiKey}</code>

        <p>
          Dieser Key wird dir nie wieder angezeigt.
        </p>

        <button onClick={() => navigate("/user")}>
          Weiter
        </button>
      </div>
    );
  }

  return (
    <div className="page-container">
      <h1>Registrieren</h1>

      <form onSubmit={handleSubmit}>
        <input
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="Name"
        />

        <input
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Email"
        />

        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Passwort"
        />

        <button>Registrieren</button>
      </form>
    </div>
  );
}
