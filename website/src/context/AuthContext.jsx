// src/context/AuthContext.jsx

import { createContext, useContext, useState, useEffect } from "react";

const AuthContext = createContext(undefined);

export function AuthProvider({ children }) {
  const [loggedIn, setLoggedIn] = useState(false);

  useEffect(() => {
    const key = localStorage.getItem("apiKey");
    if (key) {
      setLoggedIn(true);
    }
  }, []);

  function saveKey(key) {
    localStorage.setItem("apiKey", key);
    setLoggedIn(true);
  }

  function loginSuccess() {
    setLoggedIn(true);
  }

  function logout() {
    localStorage.removeItem("apiKey");
    setLoggedIn(false);
  }

  return (
    <AuthContext.Provider
      value={{
        loggedIn,
        saveKey,
        loginSuccess,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);

  if (!ctx) {
    throw new Error("useAuth must be used inside <AuthProvider>");
  }

  return ctx;
}
