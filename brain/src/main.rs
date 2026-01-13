use peppygen::subscribed_actions::my_action_robot_controller_move_left_arm as left_arm;
use peppygen::subscribed_actions::my_action_robot_controller_move_right_arm as right_arm;
use peppygen::subscribed_topics::uvc_camera_video_stream as video_stream;
use peppygen::{NodeRunner, QoSProfile, Result, run};
use std::sync::Arc;
use std::time::Duration;

async fn ai_process(node_runner: Arc<NodeRunner>) {
    loop {
        // Subscribe to video frames from the camera
        let frame_result = video_stream::on_next_message_received(&node_runner, None, None).await;

        let (_instance_id, frame) = match frame_result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to receive video frame: {e}");
                continue;
            }
        };

        // Process the frame and generate fake arm positions
        let fake_position = [
            frame.image[0] as i32,
            frame.image[1] as i32,
            frame.image[2] as i32,
        ];

        // Fire action goals to both arms concurrently
        let left_goal = left_arm::GoalRequest {
            arm_id: 0,
            desired_position: fake_position,
        };
        let right_goal = right_arm::GoalRequest {
            arm_id: 1,
            desired_position: fake_position,
        };

        let timeout = Duration::from_secs(5);

        let (left_result, right_result) = tokio::join!(
            left_arm::fire_goal(
                &node_runner,
                timeout,
                None,
                None,
                left_goal,
                QoSProfile::Standard
            ),
            right_arm::fire_goal(
                &node_runner,
                timeout,
                None,
                None,
                right_goal,
                QoSProfile::Standard
            ),
        );

        if let Err(e) = left_result {
            eprintln!("Failed to fire left arm goal: {e}");
        }
        if let Err(e) = right_result {
            eprintln!("Failed to fire right arm goal: {e}");
        }
    }
}

fn main() -> Result<()> {
    run(|_args, node_runner| async move {
        tokio::spawn(ai_process(node_runner));
        Ok(())
    })
}
