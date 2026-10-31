use pupcorn_ai::Agent;
use pupcorn_core::default_engine;

#[tokio::main]
async fn main() {
    let engine = default_engine();
    // First turn: tool call; second turn: final.
    let responses = vec![
        r#"{"type":"tool","tool":"http_probe","target":"example.com","options":{}}"#.to_string(),
        r#"{"type":"final","content":"example.com is up (HTTP 200)."}"#.to_string(),
    ];
    // MockLlm returns same response each call; use a counter-based mock instead.
    struct SeqLlm { idx: std::sync::atomic::AtomicUsize, resps: Vec<String> }
    #[async_trait::async_trait]
    impl pupcorn_ai::LlmClient for SeqLlm {
        async fn chat(&self, _m: Vec<pupcorn_ai::ChatMessage>) -> pupcorn_ai::Result<String> {
            let i = self.idx.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.resps[i % self.resps.len()].clone())
        }
    }
    let llm = SeqLlm { idx: std::sync::atomic::AtomicUsize::new(0), resps: responses };
    let agent = Agent::new(&llm, &engine);
    let answer = agent.run("probe example.com").await.unwrap();
    println!("Agent answer: {}", answer);
}