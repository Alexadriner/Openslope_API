import { useState } from "react";
import { apiFetch } from "../api/client";
import "../stylesheets/api-demo.css";

export default function ApiDemo() {
  const [data, setData] = useState({});
  const [loading, setLoading] = useState({});
  const [error, setError] = useState({});

  const fetchData = async (endpoint, key) => {
    setLoading(prev => ({ ...prev, [key]: true }));
    setError(prev => ({ ...prev, [key]: null }));
    
    try {
      const result = await apiFetch(endpoint);
      setData(prev => ({ ...prev, [key]: result }));
    } catch (err) {
      setError(prev => ({ ...prev, [key]: err.message }));
    } finally {
      setLoading(prev => ({ ...prev, [key]: false }));
    }
  };

  const apiEndpoints = [
    {
      title: "Get All Resorts",
      endpoint: "/resorts",
      key: "resorts",
      description: "Returns all ski resorts with their basic information"
    },
    {
      title: "Get Resort by ID",
      endpoint: "/resorts/kreuzberg",
      key: "resortById",
      description: "Returns a specific resort by its ID"
    },
    {
      title: "Get All Slopes",
      endpoint: "/slopes",
      key: "slopes",
      description: "Returns all slopes with their information"
    },
    {
      title: "Get All Lifts",
      endpoint: "/lifts",
      key: "lifts",
      description: "Returns all lifts with their information"
    },
    {
      title: "Get Status",
      endpoint: "/status",
      key: "status",
      description: "Returns the current API status"
    }
  ];

  return (
    <div className="api-demo-page">
      <h1>API Demo - Test Endpoints</h1>
      <p className="demo-description">
        Test the different API endpoints to see what data is available. 
        This demo shows the raw API responses from the server.
      </p>

      <div className="demo-sections">
        {apiEndpoints.map(section => (
          <div key={section.key} className="demo-section">
            <h3>{section.title}</h3>
            <p className="endpoint-description">{section.description}</p>
            <p className="endpoint-url"><strong>Endpoint:</strong> {section.endpoint}</p>
            
            <div className="demo-actions">
              <button 
                onClick={() => fetchData(section.endpoint, section.key)}
                disabled={loading[section.key]}
              >
                {loading[section.key] ? 'Loading...' : 'Test Endpoint'}
              </button>
            </div>

            {error[section.key] && (
              <div className="error-message">
                Error: {error[section.key]}
              </div>
            )}

            {data[section.key] && (
              <div className="result-preview">
                <h4>Response:</h4>
                <pre>{JSON.stringify(
                  section.key === 'resorts' 
                    ? Array.isArray(data[section.key]) 
                      ? data[section.key].slice(0, 3).concat(data[section.key].length > 3 ? ['...'] : [])
                      : data[section.key]
                    : data[section.key], 
                  null, 2
                )}</pre>
              </div>
            )}
          </div>
        ))}
      </div>

      <div className="demo-info">
        <h3>How to Use:</h3>
        <ul>
          <li>Click "Test Endpoint" to make a request to the API</li>
          <li>View the response data in the preview section</li>
          <li>Each endpoint shows different types of data from the server</li>
          <li>Use this to understand what data is available through the API</li>
        </ul>
        
        <h3>Available Endpoints:</h3>
        <ul>
          <li><strong>/resorts</strong> - Get all ski resorts</li>
          <li><strong>/resorts/:id</strong> - Get specific resort by ID</li>
          <li><strong>/slopes</strong> - Get all slopes</li>
          <li><strong>/lifts</strong> - Get all lifts</li>
          <li><strong>/status</strong> - Get API status</li>
        </ul>
      </div>
    </div>
  );
}