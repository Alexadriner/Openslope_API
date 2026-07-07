import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { apiFetch } from "../api/client";
import ResortContent from "../components/ResortContent";
import { normalizeResortName, safeDecode } from "../utils/resortData";
import "../stylesheets/base.css";
import "../stylesheets/resort-page.css";

export default function ResortPage() {
  const { name } = useParams();
  const [resort, setResort] = useState(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function loadData() {
      try {
        setError("");
        setResort(null);

        // Load first batch of resorts to find by name
        // This is necessary since the route uses resort name, not ID
        const resortsData = await apiFetch("/resorts?limit=500&offset=0");
        const targetName = normalizeResortName(safeDecode(name));
        const foundResort = resortsData.find((entry) => normalizeResortName(entry.name) === targetName);

        if (!foundResort?.id) {
          throw new Error("Resort not found.");
        }

        const resortDetail = await apiFetch(`/resorts/${foundResort.id}`);
        if (!cancelled) {
          setResort(resortDetail);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err.message || "Resort not found.");
        }
      }
    }

    loadData();

    return () => {
      cancelled = true;
    };
  }, [name]);

  if (error) {
    return <div className="page-container"><p className="error-message">{error}</p></div>;
  }

  if (!resort) {
    return <div className="page-container"><p>Loading resort...</p></div>;
  }

  return <ResortContent resort={resort} />;
}
