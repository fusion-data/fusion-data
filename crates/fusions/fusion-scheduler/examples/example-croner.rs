use croner::Cron;
use ultimate_common::time::now_local;

fn main() {
  // Parse a cron expression to find the next occurrence at 00:00 on Friday
  let cron = Cron::new("0 7 2 * * *").with_seconds_required().parse().expect("Successful parsing");

  // Get the next occurrence from the current time, excluding the current time
  let next = cron.find_next_occurrence(&now_local(), false).unwrap();

  println!("Pattern \"{}\" will match next at {}", cron.pattern.to_string(), next);
}
