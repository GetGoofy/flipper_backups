use clokwerk::{Job, Scheduler, TimeUnits};
use std::thread;
use std::time::Duration;

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler.every(1.day()).at("11:42 pm").run(|| {
        println!("Daily task at 3:20 PM");
    });

    // Run the scheduler in a loop, checking for pending tasks
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(100)); // Check every 100ms
    }
}