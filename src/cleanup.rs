use std::sync::Mutex;

static CLEANUP: Mutex<Vec<Box<dyn Fn() + Send>>> = Mutex::new(Vec::new());

/// Run all cleanup functions.
pub fn cleanup() {
    let mut cleanup = CLEANUP.lock().unwrap();
    for f in cleanup.drain(..) {
        f();
    }
}

/// Add a cleanup function to be run when the program is interrupted.
pub fn add_cleanup<F: Fn() + Send + 'static>(f: F) {
    let mut cleanup = CLEANUP.lock().unwrap();
    cleanup.push(Box::new(f));
}
