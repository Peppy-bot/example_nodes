# Example Nodes

A collection of example [Peppy](https://github.com/nicklausw/peppy) nodes demonstrating a complete robotics pipeline — from camera input, through AI decision-making, to robot arm control and video recording.

## Nodes

### fake_uvc_camera

Simulates a USB Video Class camera. Reads frames from a source video file (`assets/robot.mp4`), scales them to a configurable resolution, and publishes them on a `video_stream` topic. Also exposes a `video_stream_info` service to query camera capabilities (resolution, FPS, encoding).

### fake_robot_brain

Processes incoming video frames and sends movement commands to robot arms. Subscribes to the camera stream, performs basic processing on each frame, then fires concurrent action goals to both left and right arms via `fake_openarm01_controller`.

### fake_openarm01_controller

Controls robotic arm movements. Exposes `move_right_arm` and `move_left_arm` action servers that accept position goals, simulate smooth interpolated movement, and emit continuous feedback until the target is reached.

### fake_video_reconstruction

Records video frames from the camera and reconstructs them into an H.264-encoded MP4 file. Subscribes to the camera stream, collects frames for a configurable duration, and outputs `reconstructed_video.mp4`.

## Prerequisites

### Rust

- Rust (2024 edition)
- FFmpeg development libraries (required by `fake_uvc_camera` and `fake_video_reconstruction`)

### Python

- Python >= 3.11, < 3.14
- [uv](https://docs.astral.sh/uv/) package manager
