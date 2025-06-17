use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};

/// CORS middleware
pub async fn cors_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    // Add CORS headers
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization".parse().unwrap());

    response
}

/// Logging middleware
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::info!("Request: {} {}", method, uri);

    let response = next.run(request).await;

    tracing::info!("Response: {}", response.status());

    response
}

/// Error handling middleware
pub async fn error_handling_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    match next.run(request).await {
        response => {
            if response.status().is_server_error() {
                tracing::error!("Server error: {}", response.status());
            }
            Ok(response)
        }
    }
}

/// Authentication middleware module
pub mod auth {
    use tower::Layer;
    use tower::Service;
    use std::task::{Context, Poll};
    use std::future::Future;
    use std::pin::Pin;
    use axum::extract::Request;
    use axum::response::Response;

    /// Authentication layer
    #[derive(Clone)]
    pub struct AuthLayer;

    impl AuthLayer {
        pub fn new() -> Self {
            Self
        }
    }

    impl<S> Layer<S> for AuthLayer {
        type Service = AuthService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            AuthService { inner }
        }
    }

    /// Authentication service
    #[derive(Clone)]
    pub struct AuthService<S> {
        inner: S,
    }

    impl<S> Service<Request> for AuthService<S>
    where
        S: Service<Request, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let mut inner = self.inner.clone();
            Box::pin(async move {
                // TODO: Add authentication logic here
                inner.call(request).await
            })
        }
    }
}
