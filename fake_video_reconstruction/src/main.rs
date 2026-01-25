use std::sync::Arc;

use peppygen::subscribed_services::fake_uvc_camera_video_stream_info;
use peppygen::{NodeBuilder, NodeRunner, Parameters, Result};

fn main() -> Result<()> {
    NodeBuilder::new().run(|args: Parameters, node_runner| async move {
        let _ = args;

        tokio::spawn(poll_camera_info(node_runner));

        Ok(())
    })
}

async fn poll_camera_info(node_runner: Arc<NodeRunner>) {
    loop {
        let response = fake_uvc_camera_video_stream_info::poll(
            &node_runner,
            std::time::Duration::from_secs(5),
            None,
            None,
        )
        .await;

        match response {
            Ok(response) => {
                println!(
                    "Camera info: {}x{} @ {} fps, encoding: {}",
                    response.data.width,
                    response.data.height,
                    response.data.frames_per_second,
                    response.data.encoding
                );
                break;
            }
            Err(e) => {
                eprintln!("Failed to get camera info: {}, retrying...", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}
