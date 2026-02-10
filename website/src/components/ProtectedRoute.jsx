import { Navigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

export default function ProtectedRoute({ children }) {
  const { apiKey } = useAuth();

  if (!apiKey) {
    return <Navigate to="/login" />;
  }

  return children;
}
