use std::error::Error;
use std::future::Future;
use std::time::Duration;
use bastion_executor::prelude::*;
use lightproc::proc_stack::ProcStack;

fn describe_header() {
    println!("{}\t{}", "runtime", "time per");
}

fn describe_result(runtime: &str, time: Duration) {
    println!("{}\t{}.{:03}", runtime, time.as_secs(), time.subsec_millis());
}

// Runs the future returned by 'f()' 'count' times, in parallel.
// Returns the mean time elapsed.
async fn time_many<F, Fut, E>(f: F, count: u32) -> Result<Duration, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output=Result<(), E>>,
{
    let times = futures::future::try_join_all((0..count).map(|_|
        async {
            let start = std::time::Instant::now();
            f().await?;
            Ok(start.elapsed())
        }
    )).await?;

    Ok(times.iter().sum::<Duration>().checked_div(count).expect("count > 0"))
}

const BUF_SIZE: usize = 2048;

async fn discard<T>(_x: T) {
    // TODO: this should probably be a black-box
}

async fn concurrent_processes() -> Result<(), Box<dyn Error>> {
    let mut i = 0;
    let pid = 1;

    loop {
        let stack = ProcStack::default().with_pid(pid);
        let handle = spawn(async move {
            let stack2 = ProcStack::default().with_pid(i * 1000);
            spawn(async move {
                i * 1000
            }, stack2).await
        }, stack);
        handle.await;
        if i == 500 {
            break;
        }
        i += 1;
    }

    Ok(())
}

// Runs 'count' file-reading tasks with each fs implementation.
async fn run_benchmark<F, Fut>(func: F, name: Option<&str>, count: u32)
where
    F: Fn() -> Fut,
    Fut: Future<Output=Result<(), Box<dyn Error>>>,
{
    let time = time_many(func, count).await.expect("failed to time");

    if let Some(name) = name {
        describe_result(name, time);
    }
}

fn main() {
    env_logger::init();

    // Warm up
    run(
        run_benchmark(concurrent_processes, None, 100),
        ProcStack::default().with_after_panic(|| println!("after panic")),
    );

    describe_header();

    // Real thing
    run(
        run_benchmark(concurrent_processes, Some("bastion"), 5000),
        ProcStack::default().with_after_panic(|| println!("after panic")),
    );
}
