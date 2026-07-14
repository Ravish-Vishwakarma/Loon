use crate::backends::backend::Backend;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct WhisperCppBackend {
    model_path: String,
    binary_path: String,
}

impl WhisperCppBackend {
    pub fn new(model_path: &str, binary_path: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
            binary_path: binary_path.to_string(),
        }
    }
}

impl Backend for WhisperCppBackend {
    fn transcribe(&self, audio: &[f32]) -> Result<String, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::TempDir::new()?;
        let wav_path = temp_dir.path().join("input.wav");

        write_wav(&wav_path, audio)?;

        let binary = PathBuf::from(&self.binary_path);
        let working_dir = binary
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| temp_dir.path().to_path_buf());

        let mut cmd = Command::new(&self.binary_path);
        cmd.current_dir(&working_dir)
            .arg("-m")
            .arg(&self.model_path)
            .arg("-f")
            .arg(&wav_path)
            .arg("-otxt")
            .arg("-of")
            .arg(temp_dir.path().join("output"));

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "whisper-cli failed (exit {}): stdout={} stderr={}",
                output.status,
                stdout.trim(),
                stderr.trim()
            )
            .into());
        }

        let out_path = temp_dir.path().join("output.txt");
        let text = std::fs::read_to_string(&out_path)?;

        Ok(text.trim().to_string())
    }
}

fn write_wav(path: &Path, samples: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    for &sample in samples {
        let amplitude = (sample * 32768.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(amplitude)?;
    }

    writer.finalize()?;
    Ok(())
}
