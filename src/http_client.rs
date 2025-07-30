use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// Create an optimized HTTP client with connection pooling, keep-alive settings,
/// and appropriate timeouts for better performance and connection reuse.
pub fn create_optimized_client() -> Result<Client> {
    Ok(Client::builder()
        // Connection pooling and keep-alive settings
        .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
        .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
        .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive every 60 seconds
        
        // Timeout configurations
        .timeout(Duration::from_secs(60)) // Total request timeout
        .connect_timeout(Duration::from_secs(10)) // Connection establishment timeout
        
        // User agent for identification
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_client_creation() {
        let result = create_optimized_client();
        assert!(result.is_ok());
        
        // Verify the client has the expected configuration
        let client = result.unwrap();
        // We can't directly test the internal configuration, but we can verify it was created successfully
        assert!(format!("{:?}", client).contains("Client"));
    }

    #[test]
    fn test_multiple_optimized_clients() {
        let client1 = create_optimized_client();
        let client2 = create_optimized_client();
        
        assert!(client1.is_ok());
        assert!(client2.is_ok());
    }
}