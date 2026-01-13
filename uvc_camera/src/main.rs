use peppygen::exposed_topics::video_stream::{self, MessageHeader};
use peppygen::{Result, run};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use video_rs::Url;
use video_rs::decode::Decoder;

const FFMPEG_INSTALL_HINT: &str = "\
FFmpeg libraries are required but not found.
Please install them with:
    sudo apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev libavfilter-dev libavdevice-dev";

fn main() -> Result<()> {
    if let Err(e) = video_rs::init() {
        eprintln!("Failed to initialize video-rs: {e}\n\n{FFMPEG_INSTALL_HINT}");
        std::process::exit(1);
    }

    run(|_args, node_runner| async move {
        let node_runner = Arc::clone(&node_runner);

        tokio::spawn(async move {
            if let Err(e) = run_video_loop(node_runner).await {
                tracing::error!("Video loop error: {e:?}");
            }
        });

        Ok(())
    })
}

async fn run_video_loop(node_runner: Arc<peppygen::NodeRunner>) -> Result<()> {
    let video_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "assets", "robot.mp4"]
        .iter()
        .collect();

    if !video_path.exists() {
        panic!("Video file not found: {}", video_path.display());
    }

    let source = Url::from_file_path(&video_path).expect("Failed to create URL from path");
    let mut frame_id: u32 = 0;

    loop {
        let mut decoder = Decoder::new(&source).unwrap_or_else(|e| {
            panic!(
                "Failed to open video file '{}': {e}\n\n{FFMPEG_INSTALL_HINT}",
                video_path.display()
            )
        });
        let (width, height) = decoder.size();

        for frame in decoder.decode_iter() {
            let (_, frame) = match frame {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to decode frame: {e:?}");
                    continue;
                }
            };

            let data: Vec<u8> = frame.into_raw_vec_and_offset().0;

            let header = MessageHeader {
                stamp: SystemTime::now(),
                frame_id,
            };

            video_stream::emit(
                &node_runner,
                header,
                "rgb8".to_string(),
                width,
                height,
                data,
            )
            .await
            .expect("Failed to emit frame");

            frame_id = frame_id.wrapping_add(1);

            // Pace the emission to roughly match video framerate (assume ~30fps)
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }

        // Loop restarts - video will be reopened from the beginning
    }
}
