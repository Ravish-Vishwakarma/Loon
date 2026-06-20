pub trait Backend: Send + Sync {
    fn transcribe(&self, audio: &[f32]) -> Result<String, Box<dyn std::error::Error>>;
}
