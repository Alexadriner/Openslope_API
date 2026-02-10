// authApi.js
const API_BASE = "http://localhost:8080/";

export async function signup(username, email, password) {
  const url = `${API_BASE}signup`;

  const res = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ username, email, password }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Signup failed");
  }

  return res.json(); // enthält z.B. { api_key: "..." }
}

export async function signin(email, password) {
  const url = `${API_BASE}signin`;

  const res = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Signin failed");
  }

  return res.text(); // gibt "Login successful" zurück
}
