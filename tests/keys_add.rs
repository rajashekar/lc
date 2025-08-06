use anyhow::Result;
use lc::config::Config;
use tempfile::TempDir;

/// Temporarily override dirs::config_dir by setting HOME to a temp dir so that
/// Config::config_file_path() resolves under that temp directory.
async fn with_temp_config_env<F, Fut>(f: F) -> Result<()>
where
    F: FnOnce(&TempDir) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let temp_home = TempDir::new()?;

    // Store original env vars
    let original_home = std::env::var("HOME").ok();
    let original_xdg = std::env::var("XDG_CONFIG_HOME").ok();

    // Set HOME so dirs::config_dir() points inside temp dir on macOS/Linux
    std::env::set_var("HOME", temp_home.path());
    // Also clear XDG_* to avoid interference
    std::env::remove_var("XDG_CONFIG_HOME");

    // Run test body
    let res = f(&temp_home).await;

    // Restore original env vars
    match original_home {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }
    match original_xdg {
        Some(xdg) => std::env::set_var("XDG_CONFIG_HOME", xdg),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }

    res
}

// Mock the keys add command to avoid rpassword blocking
async fn mock_keys_add_command(
    provider_name: String,
    api_key_input: Option<String>,
) -> anyhow::Result<()> {
    let mut config = Config::load()?;
    let provider_config = match config.get_provider(&provider_name) {
        Ok(config) => config,
        Err(_) => return Err(anyhow::anyhow!("Provider '{}' not found", provider_name)),
    };

    // Simulate the Vertex AI detection and validation logic
    if provider_config
        .endpoint
        .contains("aiplatform.googleapis.com")
    {
        if let Some(b64_input) = api_key_input {
            // Decode base64 (simulating the CLI logic)
            use base64::{engine::general_purpose, Engine as _};
            let json_input = match general_purpose::STANDARD.decode(&b64_input) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(json_str) => json_str,
                    Err(_) => return Err(anyhow::anyhow!("Invalid UTF-8 in decoded base64 data")),
                },
                Err(_) => return Err(anyhow::anyhow!("Invalid base64 format")),
            };

            // Validate JSON
            let parsed: serde_json::Value = serde_json::from_str(&json_input)
                .map_err(|_| anyhow::anyhow!("Invalid JSON format"))?;

            // Check required fields
            let obj = parsed
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("JSON must be an object"))?;

            if obj.get("type").and_then(|v| v.as_str()) != Some("service_account") {
                return Err(anyhow::anyhow!(
                    "Service Account JSON must have \"type\": \"service_account\""
                ));
            }

            if !obj.contains_key("client_email") {
                return Err(anyhow::anyhow!(
                    "Service Account JSON missing 'client_email' field"
                ));
            }

            if !obj.contains_key("private_key") {
                return Err(anyhow::anyhow!(
                    "Service Account JSON missing 'private_key' field"
                ));
            }

            // Store the JSON as api_key
            config.set_api_key(provider_name.clone(), json_input)?;
            config.save()?;
        }
    }

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_keys_add_vertex_sa_json_validation_errors() -> Result<()> {
    with_temp_config_env(|_temp| async move {
        // Create a fresh Vertex provider for this test
        let mut cfg = Config::load()?;
        cfg.add_provider_with_paths(
            "vertex_validation".to_string(),
            "https://aiplatform.googleapis.com".to_string(),
            Some("/v1/models".to_string()),
            Some("https://aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/models/{model}:generateContent".to_string()),
        )?;
        cfg.save()?;

        // Verify provider was created
        let cfg_check = Config::load()?;
        cfg_check.get_provider("vertex_validation")?; // This will error if provider doesn't exist

        // Helper to test error case
        let test_error_case = |json_input: String, expected_error: String| async move {
            // Encode JSON as base64 for the test
            use base64::{Engine as _, engine::general_purpose};
            let b64_input = general_purpose::STANDARD.encode(&json_input);
            let err = mock_keys_add_command("vertex_validation".to_string(), Some(b64_input)).await.unwrap_err();
            let msg = format!("{}", err);
            assert!(msg.contains(&expected_error), "expected '{}', got: {}", expected_error, msg);
            anyhow::Ok(())
        };

        // Case A: invalid base64
        let err = mock_keys_add_command("vertex_validation".to_string(), Some("invalid_base64!@#".to_string())).await.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid base64"), "expected 'Invalid base64', got: {}", msg);

        // Case B: invalid JSON
        test_error_case("{not json".to_string(), "Invalid JSON".to_string()).await?;

        // Case C: missing type field
        test_error_case(r#"{ "client_email": "svc@proj.iam.gserviceaccount.com", "private_key": "-----BEGIN PRIVATE KEY-----\nABC\n-----END PRIVATE KEY-----\n" }"#.to_string(), "must have \"type\": \"service_account\"".to_string()).await?;

        // Case D: wrong type
        test_error_case(r#"{ "type":"user_account","client_email":"svc@proj.iam.gserviceaccount.com","private_key":"-----BEGIN PRIVATE KEY-----\nABC\n-----END PRIVATE KEY-----\n" }"#.to_string(), "must have \"type\": \"service_account\"".to_string()).await?;

        // Case E: missing client_email
        test_error_case(r#"{ "type":"service_account","private_key":"-----BEGIN PRIVATE KEY-----\nABC\n-----END PRIVATE KEY-----\n" }"#.to_string(), "missing 'client_email'".to_string()).await?;

        // Case F: missing private_key
        test_error_case(r#"{ "type":"service_account","client_email":"svc@proj.iam.gserviceaccount.com" }"#.to_string(), "missing 'private_key'".to_string()).await?;

        Ok(())
    }).await
}

#[tokio::test]
#[serial_test::serial]
async fn test_keys_add_vertex_sa_json_success_persists_full_json() -> Result<()> {
    with_temp_config_env(|_temp| async move {
        // Create Vertex provider
        {
            let mut cfg = Config::load()?;
            cfg.add_provider_with_paths(
                "vertex_success".to_string(),
                "https://aiplatform.googleapis.com".to_string(),
                Some("/v1/models".to_string()),
                Some("https://aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/models/{model}:generateContent".to_string()),
            )?;
            cfg.save()?;
        }

        let sa_json = r#"{
  "type": "service_account",
  "client_email": "svc@proj.iam.gserviceaccount.com",
  "private_key": "-----BEGIN PRIVATE KEY-----\nABC\n-----END PRIVATE KEY-----\n"
}"#;

        // Encode as base64 for the test
        use base64::{Engine as _, engine::general_purpose};
        let sa_json_b64 = general_purpose::STANDARD.encode(sa_json);
        mock_keys_add_command("vertex_success".to_string(), Some(sa_json_b64)).await?;

        // Reload config and verify api_key contains full JSON
        let cfg = Config::load()?;
        let pc = cfg.get_provider("vertex_success")?;
        let stored = pc.api_key.as_ref().expect("api_key should be set");
        assert_eq!(stored, sa_json);

        Ok(())
    }).await
}
