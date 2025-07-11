use axum::response::Html;

use crate::error::Result;

pub async fn api_docs() -> Result<Html<&'static str>> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Nagari Registry API Documentation</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .endpoint { margin: 20px 0; padding: 15px; border: 1px solid #ddd; }
        .method { display: inline-block; padding: 4px 8px; color: white; border-radius: 3px; }
        .get { background-color: #61affe; }
        .post { background-color: #49cc90; }
        .put { background-color: #fca130; }
        .delete { background-color: #f93e3e; }
    </style>
</head>
<body>
    <h1>Nagari Registry API Documentation</h1>

    <h2>Package Endpoints</h2>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/packages</code>
        <p>List all packages with pagination and sorting options.</p>
        <strong>Query Parameters:</strong>
        <ul>
            <li><code>page</code> - Page number (default: 1)</li>
            <li><code>per_page</code> - Items per page (default: 20, max: 100)</li>
            <li><code>sort</code> - Sort field (default: updated_at)</li>
            <li><code>order</code> - Sort order: asc/desc (default: desc)</li>
        </ul>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/packages/{name}</code>
        <p>Get package information by name.</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/packages/{name}/{version}</code>
        <p>Get specific package version information.</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/packages/{name}/{version}/download</code>
        <p>Download package tarball.</p>
    </div>

    <div class="endpoint">
        <span class="method post">POST</span> <code>/packages</code>
        <p>Publish a new package version (requires authentication).</p>
        <strong>Content-Type:</strong> multipart/form-data
        <ul>
            <li><code>metadata</code> - Package metadata JSON</li>
            <li><code>tarball</code> - Package tarball file</li>
        </ul>
    </div>

    <div class="endpoint">
        <span class="method delete">DELETE</span> <code>/packages/{name}</code>
        <p>Delete entire package (requires authentication and ownership).</p>
    </div>

    <div class="endpoint">
        <span class="method delete">DELETE</span> <code>/packages/{name}/{version}</code>
        <p>Delete specific package version (requires authentication and ownership).</p>
    </div>

    <h2>User Endpoints</h2>

    <div class="endpoint">
        <span class="method post">POST</span> <code>/users/register</code>
        <p>Register a new user account.</p>
    </div>

    <div class="endpoint">
        <span class="method post">POST</span> <code>/users/login</code>
        <p>Login and receive authentication token.</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/users/profile</code>
        <p>Get current user profile (requires authentication).</p>
    </div>

    <div class="endpoint">
        <span class="method put">PUT</span> <code>/users/profile</code>
        <p>Update current user profile (requires authentication).</p>
    </div>

    <h2>Search Endpoints</h2>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/search</code>
        <p>Search for packages.</p>
        <strong>Query Parameters:</strong>
        <ul>
            <li><code>q</code> - Search query (required)</li>
            <li><code>page</code> - Page number (default: 1)</li>
            <li><code>per_page</code> - Items per page (default: 20, max: 100)</li>
            <li><code>sort</code> - Sort field (default: relevance)</li>
        </ul>
    </div>

    <h2>Statistics Endpoints</h2>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/stats</code>
        <p>Get registry statistics.</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/packages/{name}/stats</code>
        <p>Get statistics for a specific package.</p>
    </div>

    <h2>System Endpoints</h2>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/health</code>
        <p>Health check endpoint.</p>
    </div>

    <div class="endpoint">
        <span class="method get">GET</span> <code>/docs</code>
        <p>This API documentation.</p>
    </div>

    <h2>Authentication</h2>
    <p>
        Authentication is required for publishing packages and accessing user endpoints.
        Include the JWT token in the Authorization header: <code>Authorization: Bearer &lt;token&gt;</code>
    </p>
</body>
</html>
    "#;

    Ok(Html(html))
}
