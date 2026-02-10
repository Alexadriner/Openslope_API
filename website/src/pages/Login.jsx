import { useState } from "react";
import { signin } from "../api/auth";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const { loginSuccess } = useAuth();
  const navigate = useNavigate();

  async function handleSubmit(e) {
    e.preventDefault();

    try {
      await signin(email, password);

      // Nur Status setzen
      loginSuccess();

      navigate("/user");
    } catch (err) {
      alert(err);
    }
  }

  return (
    <div className="page-container">
      <h1>Login</h1>

      <form onSubmit={handleSubmit}>
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

        <button>Login</button>
      </form>
    </div>
  );
}
