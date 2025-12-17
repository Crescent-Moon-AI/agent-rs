//! Example demonstrating OpenAI provider with custom configuration
//!
//! This example shows how to:
//! - Configure custom API base URL (for OpenAI-compatible APIs)
//! - Set custom timeout
//! - Configure supported models list
//! - Use with local LLM deployments

use agent_llm::providers::{OpenAIConfig, OpenAIProvider};

// Uncomment these if you enable the request example at the end
// use agent_llm::{CompletionRequest, LLMProvider, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OpenAI Provider Custom Configuration Examples ===\n");

    // Example 1: Basic configuration with default OpenAI API
    println!("1. Basic configuration with OpenAI API:");
    let basic_config = OpenAIConfig::new("your-api-key-here")
        .with_timeout(60); // 60 second timeout

    let provider = OpenAIProvider::with_config(basic_config)?;
    println!("   API Base: {}", provider.config().api_base);
    println!("   Timeout: {} seconds", provider.config().timeout_secs);
    println!();

    // Example 2: Configuration with model validation
    println!("2. Configuration with supported models:");
    let validated_config = OpenAIConfig::new("your-api-key-here")
        .with_supported_models(vec![
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]);

    let validated_provider = OpenAIProvider::with_config(validated_config)?;
    println!("   Supported models: {:?}", validated_provider.config().supported_models);
    println!();

    // Example 3: Local LLM deployment (e.g., llama.cpp, vLLM, text-generation-webui)
    println!("3. Local LLM deployment configuration:");
    let local_config = OpenAIConfig::new("not-needed") // Many local APIs don't need auth
        .with_api_base("http://localhost:8000/v1")
        .with_timeout(180); // Longer timeout for local models

    let local_provider = OpenAIProvider::with_config(local_config)?;
    println!("   API Base: {}", local_provider.config().api_base);
    println!("   This configuration works with:");
    println!("   - llama.cpp server");
    println!("   - vLLM");
    println!("   - text-generation-webui");
    println!("   - LocalAI");
    println!("   - Ollama (with OpenAI compatibility)");
    println!();

    // Example 4: Azure OpenAI configuration
    println!("4. Azure OpenAI configuration:");
    let azure_config = OpenAIConfig::new("your-azure-api-key")
        .with_api_base("https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT");

    let azure_provider = OpenAIProvider::with_config(azure_config)?;
    println!("   API Base: {}", azure_provider.config().api_base);
    println!();

    // Example 5: Adding models incrementally
    println!("5. Building model list incrementally:");
    let incremental_config = OpenAIConfig::new("your-api-key-here")
        .add_supported_model("gpt-4-turbo")
        .add_supported_model("gpt-4")
        .add_supported_model("custom-model-v1");

    let incremental_provider = OpenAIProvider::with_config(incremental_config)?;
    println!("   Supported models: {:?}", incremental_provider.config().supported_models);
    println!();

    // Example 6: Reading from environment variables
    println!("6. Configuration from environment:");
    println!("   Set these environment variables:");
    println!("   - OPENAI_API_KEY: Your API key");
    println!("   - OPENAI_API_BASE: (Optional) Custom API base URL");
    println!();
    println!("   Then use:");
    println!("   let provider = OpenAIProvider::from_env()?;");
    println!();

    // Example 7: Using the configured provider (if you have an API key)
    // Uncomment this section to actually make a request:
    /*
    println!("7. Making a request with custom configuration:");

    let config = OpenAIConfig::from_env()?; // Or use any config from above
    let provider = OpenAIProvider::with_config(config)?;

    let request = CompletionRequest::builder("gpt-3.5-turbo")
        .add_message(Message::user("What is the capital of France?"))
        .max_tokens(50)
        .temperature(0.7)
        .build();

    match provider.complete(request).await {
        Ok(response) => {
            println!("   Response: {}", response.message.text().unwrap_or("No text"));
            println!("   Tokens used: {} input + {} output = {} total",
                response.usage.input_tokens,
                response.usage.output_tokens,
                response.usage.total());
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    */

    println!("=== Configuration Complete ===");
    println!("\nNote: To make actual requests, uncomment the example code");
    println!("and provide a valid API key or configure a local deployment.");

    Ok(())
}
