use peppygen::exposed_actions::{move_left_arm, move_right_arm};
use peppygen::{NodeBuilder, Parameters, Result};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;

fn main() -> Result<()> {
    NodeBuilder::<Parameters>::new().run(|_args, node_runner| async move {
        let left_runner = Arc::clone(&node_runner);
        let right_runner = Arc::clone(&node_runner);
        let left_cancel_token = node_runner.cancellation_token().clone();
        let right_cancel_token = node_runner.cancellation_token().clone();

        tokio::spawn(async move {
            if let Err(error) = run_left_arm_action(left_runner, left_cancel_token).await {
                tracing::error!("Left arm action error: {error:?}");
            }
        });

        tokio::spawn(async move {
            if let Err(error) = run_right_arm_action(right_runner, right_cancel_token).await {
                tracing::error!("Right arm action error: {error:?}");
            }
        });

        Ok(())
    })
}

async fn run_left_arm_action(
    node_runner: Arc<peppygen::NodeRunner>,
    cancel_token: CancellationToken,
) -> Result<()> {
    println!("[controller] Left arm action handler started");
    let mut action = move_left_arm::ActionHandle::expose(&node_runner).await?;
    let mut last_position = [0, 0, 0];

    loop {
        if cancel_token.is_cancelled() {
            println!("[controller] Left arm shutdown requested");
            break;
        }

        let Some(goal_request) = wait_for_left_goal(&mut action).await? else {
            println!("[controller] Left arm action handler closed");
            break;
        };

        let desired_position = goal_request.data.desired_position;
        println!(
            "[controller] Left arm received goal: {:?}",
            desired_position
        );
        let duration = choose_action_duration();

        let outcome =
            execute_left_arm_goal(&mut action, last_position, desired_position, duration).await?;

        let final_position = match outcome {
            ActionOutcome::Completed(position) => {
                println!(
                    "[controller] Left arm completed at position: {:?}",
                    position
                );
                last_position = position;
                position
            }
            ActionOutcome::Cancelled(position) => {
                println!(
                    "[controller] Left arm cancelled at position: {:?}",
                    position
                );
                last_position = position;
                position
            }
            ActionOutcome::Closed => {
                println!("[controller] Left arm action closed");
                break;
            }
        };

        let handled = action
            .handle_result_next_request(move |_request| {
                Ok(move_left_arm::ResultResponse::new(final_position))
            })
            .await?;

        if !handled {
            break;
        }
    }

    Ok(())
}

async fn run_right_arm_action(
    node_runner: Arc<peppygen::NodeRunner>,
    cancel_token: CancellationToken,
) -> Result<()> {
    println!("[controller] Right arm action handler started");
    let mut action = move_right_arm::ActionHandle::expose(&node_runner).await?;
    let mut last_position = [0, 0, 0];

    loop {
        if cancel_token.is_cancelled() {
            println!("[controller] Right arm shutdown requested");
            break;
        }

        let Some(goal_request) = wait_for_right_goal(&mut action).await? else {
            println!("[controller] Right arm action handler closed");
            break;
        };

        let desired_position = goal_request.data.desired_position;
        println!(
            "[controller] Right arm received goal: {:?}",
            desired_position
        );
        let duration = choose_action_duration();

        let outcome =
            execute_right_arm_goal(&mut action, last_position, desired_position, duration).await?;

        let final_position = match outcome {
            ActionOutcome::Completed(position) => {
                println!(
                    "[controller] Right arm completed at position: {:?}",
                    position
                );
                last_position = position;
                position
            }
            ActionOutcome::Cancelled(position) => {
                println!(
                    "[controller] Right arm cancelled at position: {:?}",
                    position
                );
                last_position = position;
                position
            }
            ActionOutcome::Closed => {
                println!("[controller] Right arm action closed");
                break;
            }
        };

        let handled = action
            .handle_result_next_request(move |_request| {
                Ok(move_right_arm::ResultResponse::new(final_position))
            })
            .await?;

        if !handled {
            break;
        }
    }

    Ok(())
}

async fn wait_for_left_goal(
    action: &mut move_left_arm::ActionHandle,
) -> Result<Option<move_left_arm::GoalRequest>> {
    let goal_holder = Arc::new(Mutex::new(None));
    let goal_holder_clone = Arc::clone(&goal_holder);

    let handled = action
        .handle_goal_next_request(move |request| {
            *goal_holder_clone.lock().expect("left goal lock poisoned") = Some(request);
            Ok(move_left_arm::GoalResponse::new(true))
        })
        .await?;

    if !handled {
        return Ok(None);
    }

    Ok(goal_holder.lock().expect("left goal lock poisoned").take())
}

async fn wait_for_right_goal(
    action: &mut move_right_arm::ActionHandle,
) -> Result<Option<move_right_arm::GoalRequest>> {
    let goal_holder = Arc::new(Mutex::new(None));
    let goal_holder_clone = Arc::clone(&goal_holder);

    let handled = action
        .handle_goal_next_request(move |request| {
            *goal_holder_clone.lock().expect("right goal lock poisoned") = Some(request);
            Ok(move_right_arm::GoalResponse::new(true))
        })
        .await?;

    if !handled {
        return Ok(None);
    }

    Ok(goal_holder.lock().expect("right goal lock poisoned").take())
}

fn choose_action_duration() -> Duration {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or_default();
    let millis = 1_000 + (nanos % 2_000) as u64;
    Duration::from_millis(millis)
}

async fn execute_left_arm_goal(
    action: &mut move_left_arm::ActionHandle,
    start: [i32; 3],
    target: [i32; 3],
    duration: Duration,
) -> Result<ActionOutcome> {
    action.emit_feedback(start).await?;

    match poll_left_cancel(action).await? {
        CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(start)),
        CancelPoll::Closed => return Ok(ActionOutcome::Closed),
        CancelPoll::None => {}
    }

    let (steps, step_duration) = feedback_plan(duration);
    let mut current = start;

    for step in 1..=steps {
        tokio::time::sleep(step_duration).await;

        match poll_left_cancel(action).await? {
            CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(current)),
            CancelPoll::Closed => return Ok(ActionOutcome::Closed),
            CancelPoll::None => {}
        }

        let ratio = step as f32 / steps as f32;
        current = interpolate_position(start, target, ratio);
        action.emit_feedback(current).await?;

        match poll_left_cancel(action).await? {
            CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(current)),
            CancelPoll::Closed => return Ok(ActionOutcome::Closed),
            CancelPoll::None => {}
        }
    }

    Ok(ActionOutcome::Completed(target))
}

async fn execute_right_arm_goal(
    action: &mut move_right_arm::ActionHandle,
    start: [i32; 3],
    target: [i32; 3],
    duration: Duration,
) -> Result<ActionOutcome> {
    action.emit_feedback(start).await?;

    match poll_right_cancel(action).await? {
        CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(start)),
        CancelPoll::Closed => return Ok(ActionOutcome::Closed),
        CancelPoll::None => {}
    }

    let (steps, step_duration) = feedback_plan(duration);
    let mut current = start;

    for step in 1..=steps {
        tokio::time::sleep(step_duration).await;

        match poll_right_cancel(action).await? {
            CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(current)),
            CancelPoll::Closed => return Ok(ActionOutcome::Closed),
            CancelPoll::None => {}
        }

        let ratio = step as f32 / steps as f32;
        current = interpolate_position(start, target, ratio);
        action.emit_feedback(current).await?;

        match poll_right_cancel(action).await? {
            CancelPoll::Cancelled => return Ok(ActionOutcome::Cancelled(current)),
            CancelPoll::Closed => return Ok(ActionOutcome::Closed),
            CancelPoll::None => {}
        }
    }

    Ok(ActionOutcome::Completed(target))
}

fn feedback_plan(duration: Duration) -> (u32, Duration) {
    let total_ms = duration.as_millis().max(1);
    let steps = (total_ms / 200).max(1) as u32;
    let step_ms = (total_ms / steps as u128).max(1) as u64;
    (steps, Duration::from_millis(step_ms))
}

fn interpolate_position(start: [i32; 3], target: [i32; 3], ratio: f32) -> [i32; 3] {
    [
        lerp_i32(start[0], target[0], ratio),
        lerp_i32(start[1], target[1], ratio),
        lerp_i32(start[2], target[2], ratio),
    ]
}

fn lerp_i32(start: i32, target: i32, ratio: f32) -> i32 {
    let delta = (target - start) as f32;
    (start as f32 + delta * ratio).round() as i32
}

#[derive(Debug, Clone, Copy)]
enum ActionOutcome {
    Completed([i32; 3]),
    Cancelled([i32; 3]),
    Closed,
}

#[derive(Debug, Clone, Copy)]
enum CancelPoll {
    None,
    Cancelled,
    Closed,
}

async fn poll_left_cancel(action: &mut move_left_arm::ActionHandle) -> Result<CancelPoll> {
    match tokio::time::timeout(
        Duration::from_millis(0),
        action.handle_cancel_next_request(|_request| {
            Ok(move_left_arm::CancelResponse::new(true, None))
        }),
    )
    .await
    {
        Ok(result) => match result? {
            true => Ok(CancelPoll::Cancelled),
            false => Ok(CancelPoll::Closed),
        },
        Err(_) => Ok(CancelPoll::None),
    }
}

async fn poll_right_cancel(action: &mut move_right_arm::ActionHandle) -> Result<CancelPoll> {
    match tokio::time::timeout(
        Duration::from_millis(0),
        action.handle_cancel_next_request(|_request| {
            Ok(move_right_arm::CancelResponse::new(true, None))
        }),
    )
    .await
    {
        Ok(result) => match result? {
            true => Ok(CancelPoll::Cancelled),
            false => Ok(CancelPoll::Closed),
        },
        Err(_) => Ok(CancelPoll::None),
    }
}
