use reqwest::Client;
use serde_json::json;

pub async fn query_local_agent(analysis_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let prompt = format!(
        "You are an EVM security agent. Analyze this transaction pre-execution structural analysis:\n{}\nBriefly assess risk and give a PASS or WARN recommendation.",
        analysis_json
    );

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&json!({
            "model": "smollm2:135m",
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await?;

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await?;
        let response = body["response"].as_str().unwrap_or("No response generated.").to_string();
        Ok(response)
    } else {
        Ok("Local LLM model smollm2:135m unavailable or Ollama offline.".to_string())
    }
}