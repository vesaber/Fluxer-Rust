use std::sync::Arc;
use livekit::options::TrackPublishOptions;
use livekit::track::{LocalAudioTrack, LocalTrack, TrackSource};
use livekit::webrtc::audio_source::native::NativeAudioSource;
use livekit::webrtc::prelude::*;
use livekit::Room;
use std::process::Stdio;
use tokio::io::AsyncReadExt as _;
use tokio::process::Command;
use crate::http::Http;
use tokio::task::AbortHandle;

pub struct FluxerVoiceConnection {
    pub room: Arc<Room>,
    audio_source: NativeAudioSource,
}

impl FluxerVoiceConnection {
    pub async fn connect(
        url: &str,
        token: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (room, events) = Room::connect(url, token, Default::default()).await?;
        let room = Arc::new(room);
        tokio::spawn(async move { let mut e = events; while e.recv().await.is_some() {} });
        let source = NativeAudioSource::new(Default::default(), 48_000, 2, 960);

        let track = LocalAudioTrack::create_audio_track(
            "audio",
            livekit::webrtc::audio_source::RtcAudioSource::Native(source.clone()),
        );

        room.local_participant()
            .publish_track(
                LocalTrack::Audio(track),
                TrackPublishOptions {
                    source: TrackSource::Microphone,
                    ..Default::default()
                },
            )
            .await?;

        Ok(Self { room, audio_source: source })
    }

    pub async fn play_music(
        &self,
        path: &str,
        http: Arc<Http>,
        channel_id: String,
    ) -> Result<AbortHandle, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new("ffmpeg")
            .args(["-re", "-i", path, "-f", "s16le", "-ar", "48000", "-ac", "2", "pipe:1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = child.stdout.take().ok_or("ffmpeg: no stdout")?;
        let mut stderr = child.stderr.take().ok_or("ffmpeg: no stderr")?;
        let source = self.audio_source.clone();

        let handle = tokio::spawn(async move {
            let mut buffer = vec![0u8; 960 * 2 * 2];
            let mut stream_error: Option<String> = None;

            loop {
                match stdout.read_exact(&mut buffer).await {
                    Ok(_) => {
                        let samples: Vec<i16> = buffer
                            .chunks_exact(2)
                            .map(|c| i16::from_le_bytes([c[0], c[1]]))
                            .collect();

                        if let Err(e) = source.capture_frame(&AudioFrame {
                            data: samples.into(),
                            num_channels: 2,
                            sample_rate: 48_000,
                            samples_per_channel: 960,
                        }).await {
                            stream_error = Some(format!("Audio capture error: {}", e));
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::UnexpectedEof {
                            stream_error = Some(format!("PCM read error: {}", e));
                        }
                        break;
                    }
                }
            }

            let exit_status = child.wait().await;
            let failed = exit_status.map(|s| !s.success()).unwrap_or(true);
            if failed || stream_error.is_some() {
                let mut stderr_output = String::new();
                let _ = stderr.read_to_string(&mut stderr_output).await;

                let last_lines: String = stderr_output
                    .lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n");

                let error_msg = stream_error.unwrap_or_else(|| {
                    format!("ffmpeg exited with an error:\n```\n{}\n```", last_lines)
                });

                let _ = http.send_message(&channel_id, &error_msg).await;
            }
        });

        Ok(handle.abort_handle())
    }
}