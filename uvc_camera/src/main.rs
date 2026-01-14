use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context as ScalerContext, Flags as ScalerFlags};
use ffmpeg::util::frame::video::Video as VideoFrame;
use ffmpeg_next as ffmpeg;
use peppygen::exposed_topics::video_stream::{self, MessageHeader};
use peppygen::parameters;
use peppygen::{Result, run};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

fn main() -> Result<()> {
    ffmpeg::init().expect("Failed to initialize FFmpeg");

    run(|args, node_runner| async move {
        let node_runner = Arc::clone(&node_runner);
        let video_params = args.video.clone();

        tokio::spawn(async move {
            if let Err(e) = run_video_loop(node_runner, video_params).await {
                tracing::error!("Video loop error: {e:?}");
            }
        });

        Ok(())
    })
}

async fn run_video_loop(
    node_runner: Arc<peppygen::NodeRunner>,
    video_params: parameters::video::Video,
) -> Result<()> {
    println!("[uvc_camera] Starting video loop...");
    let video_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "assets", "robot.mp4"]
        .iter()
        .collect();

    if !video_path.exists() {
        panic!("Video file not found: {}", video_path.display());
    }
    println!("[uvc_camera] Video file found: {}", video_path.display());

    let mut frame_id: u32 = 0;

    let width = video_params.resolution.width as u32;
    let height = video_params.resolution.height as u32;
    let encoding = video_params.encoding.clone();
    let frame_duration_ms = 1000 / video_params.frame_rate as u64;

    loop {
        println!("[uvc_camera] Opening video file for playback...");
        let mut input = ffmpeg::format::input(&video_path).unwrap_or_else(|e| {
            panic!("Failed to open video file '{}': {e}", video_path.display())
        });

        let video_stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .expect("No video stream found");
        let video_stream_index = video_stream.index();

        let context_decoder = ffmpeg::codec::Context::from_parameters(video_stream.parameters())
            .expect("Failed to create codec context");
        let mut decoder = context_decoder
            .decoder()
            .video()
            .expect("Failed to create video decoder");

        let mut scaler = ScalerContext::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            width,
            height,
            ScalerFlags::BILINEAR,
        )
        .expect("Failed to create scaler");

        let mut receive_and_emit_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> std::result::Result<(), ffmpeg::Error> {
                let mut decoded_frame = VideoFrame::empty();
                while decoder.receive_frame(&mut decoded_frame).is_ok() {
                    let mut rgb_frame = VideoFrame::empty();
                    scaler.run(&decoded_frame, &mut rgb_frame)?;

                    let data: Vec<u8> = rgb_frame.data(0).to_vec();

                    let header = MessageHeader {
                        stamp: SystemTime::now(),
                        frame_id,
                    };

                    // Use blocking emit since we're in a sync closure
                    let node_runner = Arc::clone(&node_runner);
                    let encoding = encoding.clone();
                    let current_frame_id = frame_id;
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            video_stream::emit(&node_runner, header, encoding, width, height, data)
                                .await
                                .expect("Failed to emit frame");
                        });
                    });
                    println!("[uvc_camera] Emitted frame {}", current_frame_id);

                    frame_id = frame_id.wrapping_add(1);

                    std::thread::sleep(std::time::Duration::from_millis(frame_duration_ms));
                }
                Ok(())
            };

        for (stream, packet) in input.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet).ok();
                receive_and_emit_frames(&mut decoder).ok();
            }
        }

        // Flush the decoder
        decoder.send_eof().ok();
        receive_and_emit_frames(&mut decoder).ok();

        // Loop restarts - video will be reopened from the beginning
        println!("[uvc_camera] Video ended, restarting from beginning...");
    }
}
