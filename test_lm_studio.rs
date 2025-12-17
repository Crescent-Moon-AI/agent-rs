//! Simple test to verify LM Studio connection with OpenAI provider

use agent_llm::{CompletionRequest, LLMProvider, Message};
use agent_llm::providers::{OpenAIConfig, OpenAIProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing LM Studio Connection ===\n");

    // Configure OpenAI provider for LM Studio (based on screenshot)
    let config = OpenAIConfig::new("lm-studio")
        .with_api_base("http://198.18.0.1:22222/v1")
        .with_timeout(180);

    println!("Configuration:");
    println!("  API Base: {}", config.api_base);
    println!("  Timeout: {}s\n", config.timeout_secs);

    // Create provider
    let provider = OpenAIProvider::with_config(config)?;
    println!("Provider created: {}\n", provider.name());

    // Simple test request
    println!("Sending test request to LM Studio...");
    let request = CompletionRequest::builder("qwen/qwen3-vl-8b")
        .add_message(Message::user("Hello! Please respond with a brief greeting."))
        .max_tokens(100)
        .temperature(0.7)
        .build();

    match provider.complete(request).await {
        Ok(response) => {
            println!("\n✓ Success!");
            println!("Response: {}", response.message.text().unwrap_or("No text"));
            println!("\nToken usage:");
            println!("  Input:  {}", response.usage.input_tokens);
            println!("  Output: {}", response.usage.output_tokens);
            println!("  Total:  {}", response.usage.total());
            println!("\nStop reason: {:?}", response.stop_reason);
        }
        Err(e) => {
            println!("\n✗ Error!");
            println!("Failed to connect to LM Studio: {}", e);
            println!("\nPlease verify:");
            println!("  1. LM Studio is running");
            println!("  2. Server is accessible at http://198.18.0.1:22222");
            println!("  3. A model is loaded (qwen/qwen3-vl-8b or similar)");
            return Err(e.into());
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
