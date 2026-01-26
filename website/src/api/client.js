const API_BASE = "http://localhost:8080/";

// API-Key hier eintragen
const API_KEY = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA";

export async function apiFetch(path, options = {}) {
  // URL mit API-Key als Query-Parameter
  const url = new URL(`${API_BASE}${path}`);
  url.searchParams.append("api_key", API_KEY);

  const res = await fetch(url.toString(), {
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
    ...options,
  });

  if (!res.ok) {
    throw new Error("API error");
  }

  return res.json();
}