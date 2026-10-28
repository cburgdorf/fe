//! Runs a batch of inputs under a per-input deadline, so that an input which
//! makes the code under test loop fails the test instead of stalling the suite.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

/// Feeds each input, in order, to the function that `make` builds.
///
/// The function runs on a worker thread. A panic is recorded and the function
/// is built again, since its state may be corrupted. All panics are reported
/// together at the end, each with its input.
///
/// An input that gives no result within `deadline` fails the test at once and
/// names that input. A thread cannot be stopped, so the stuck worker keeps
/// running until the test process exits.
pub fn run_inputs<F>(
    inputs: impl IntoIterator<Item = String>,
    deadline: Duration,
    make: impl Fn() -> F + Send + 'static,
) where
    F: FnMut(&str),
{
    let (input_tx, input_rx) = mpsc::channel::<String>();
    let (result_tx, result_rx) = mpsc::channel::<Result<(), String>>();

    thread::spawn(move || {
        let mut run = make();
        for input in input_rx {
            let result = catch_unwind(AssertUnwindSafe(|| run(&input))).map_err(panic_message);
            if result.is_err() {
                run = make();
            }
            if result_tx.send(result).is_err() {
                return;
            }
        }
    });

    let mut panics = Vec::new();
    for input in inputs {
        input_tx.send(input.clone()).expect("worker thread exited");
        match result_rx.recv_timeout(deadline) {
            Ok(Ok(())) => {}
            Ok(Err(message)) => panics.push(format!("panic on {input:?}: {message}")),
            Err(RecvTimeoutError::Timeout) => {
                panic!("no result after {deadline:?} for input {input:?}, which probably loops")
            }
            Err(RecvTimeoutError::Disconnected) => panic!("worker thread exited"),
        }
    }

    assert!(
        panics.is_empty(),
        "{} input(s) panicked:\n{}",
        panics.len(),
        panics.join("\n")
    );
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<non-string panic>")
        .to_owned()
}
