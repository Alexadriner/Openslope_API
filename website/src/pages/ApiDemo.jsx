import "../stylesheets/base.css";

export default function ApiDemo() {
  return (
    <div className="page-container">
      <h1>API Demo</h1>
      <p>Test our SkiAPI directly in the browser.</p>

      <label>
        Endpoint:
        <select>
          <option>/api/v1/resorts</option>
          <option>/api/v1/resorts/:id</option>
          <option>/api/v1/search</option>
        </select>
      </label>

      <br />
      <br />

      <button>Send request</button>

      <h3>Response</h3>
      <pre>{"{ }"}</pre>
    </div>
  );
}
