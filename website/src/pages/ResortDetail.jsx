import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { apiFetch } from "../api/client";
import ResortContent from "../components/ResortContent";
import "../stylesheets/base.css";
import "../stylesheets/resort-page.css";

export default function ResortDetail() {
  const { id } = useParams();
  const [resort, setResort] = useState(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function loadResort() {
      try {
        setError("");
        setResort(null);
        const response = await apiFetch(`/resorts/${id}`);
        if (!cancelled) {
          setResort(response);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err.message || "Resort could not be loaded.");
        }
      }
    }

    loadResort();

    return () => {
      cancelled = true;
    };
  }, [id]);

  if (error) {
    return <div className="page-container"><p className="error-message">{error}</p></div>;
  }

  if (!resort) {
    return <div className="page-container"><p>Loading resort...</p></div>;
  }

  return <ResortContent resort={resort} />;
}
