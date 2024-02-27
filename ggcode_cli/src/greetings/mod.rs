use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

pub fn generate_wishes() -> String {
    let wishes_options: Vec<&str> = vec![
        "Take care.",
        "Have a nice day!",
        "Ciao.",
        "Adios.",
        "Good seeing you!",
        "Bye for now.",
        "See you around.",
        "Have fun!",
        "Keep in touch.",
        "Goodbye."
    ];

    let v: usize = rand::random();
    let st: &str = wishes_options[v % wishes_options.len()];
    return st.to_string();
}

pub fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    return pb;
}
