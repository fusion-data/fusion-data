use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use ultimate_common::time;

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
  let mut sched = JobScheduler::new().await?;
  let tz = *time::local_offset();
  println!("Local time zone is {:?}", tz);

  // Add basic cron job
  //   sched
  //     .add(Job::new_tz("1/10 * * * * *", tz, |job_id, _l| {
  //       println!("I run every 10 seconds, job id is {}", job_id);
  //     })?)
  //     .await?;

  //   // Add async job
  //   sched
  //     .add(Job::new_async_tz("1/7 * * * * *", tz, |job_id, mut l| {
  //       Box::pin(async move {
  //         println!("I run async every 7 seconds");

  //         // Query the next execution time for this job
  //         let next_tick = l.next_tick_for_job(job_id).await;
  //         match next_tick {
  //           Ok(Some(ts)) => println!("Next time for 7s job is {:?}", time::to_local(ts)),
  //           _ => println!("Could not get next tick for 7s job"),
  //         }
  //       })
  //     })?)
  //     .await?;

  //   // Add one-shot job with given duration
  //   sched
  //     .add(Job::new_one_shot(Duration::from_secs(18), |job_id, _l| {
  //       println!("I only run once, job id is {}", job_id);
  //     })?)
  //     .await?;

  // Create repeated job with given duration, make it mutable to edit it afterwards
  let mut jj = Job::new_repeated(Duration::from_secs(8), |uuid, _l| {
    println!("I run repeatedly every 8 seconds, job id is {}", uuid);
  })?;

  // Add actions to be executed when the jobs starts/stop etc.
  let notification_start_id = jj
    .on_start_notification_add(
      &sched,
      Box::new(|job_id, notification_id, type_of_notification| {
        Box::pin(async move {
          println!("Job {:?} was started, notification {:?} ran ({:?})", job_id, notification_id, type_of_notification);
        })
      }),
    )
    .await?;
  println!("Notification start id is {}", notification_start_id);

  jj.on_stop_notification_add(
    &sched,
    Box::new(|job_id, notification_id, type_of_notification| {
      Box::pin(async move {
        println!("Job {:?} was completed, notification {:?} ran ({:?})", job_id, notification_id, type_of_notification);
      })
    }),
  )
  .await?;

  jj.on_removed_notification_add(
    &sched,
    Box::new(|job_id, notification_id, type_of_notification| {
      Box::pin(async move {
        println!("Job {:?} was removed, notification {:?} ran ({:?})", job_id, notification_id, type_of_notification);
      })
    }),
  )
  .await?;

  let jj_job_id = sched.add(jj).await?;
  println!("Job id is {}", jj_job_id);

  // Feature 'signal' must be enabled
  sched.shutdown_on_ctrl_c();

  // Add code to be run during/after shutdown
  sched.set_shutdown_handler(Box::new(|| {
    Box::pin(async move {
      println!("Shut down done");
    })
  }));

  // Start the scheduler
  sched.start().await?;

  // Wait while the jobs run
  tokio::time::sleep(Duration::from_secs(20)).await;

  sched.remove(&jj_job_id).await?;
  println!("Removed job[{}] successfully", jj_job_id);

  tokio::time::sleep(Duration::from_secs(5)).await;
  sched.shutdown().await?;

  Ok(())
}
