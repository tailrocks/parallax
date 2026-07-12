#[cfg(test)]
mod tests {
    use std::{fs, thread, time::Duration};

    #[test]
    fn pass() {}

    #[test]
    fn retry_pass() {
        let state = std::env::var("PARALLAX_NEXTEST_FIXTURE_STATE").expect("state path");
        if !std::path::Path::new(&state).exists() {
            fs::write(state, "first attempt failed").expect("state marker");
            panic!("intentional first-attempt failure");
        }
    }

    #[test]
    fn persistent_fail() {
        panic!("intentional persistent failure");
    }

    #[test]
    fn slow_pass() {
        thread::sleep(Duration::from_millis(250));
    }

    #[test]
    fn timeout() {
        thread::sleep(Duration::from_secs(2));
    }
}
