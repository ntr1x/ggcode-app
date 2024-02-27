// use indicatif::ProgressBar;
//
// use ggcode_core::generator::{Progress, ProgressFactory};
//
// use crate::greetings::create_progress_bar;
//
// pub struct AppProgressFactory;
//
// pub struct AppProgress {
//     pg: ProgressBar,
// }
//
// impl Progress for AppProgress {
//     fn progress(&self, msg: &String) {
//         &self.pg.set_message(msg.clone());
//     }
//
//     fn finish(&self, msg: &String) {
//         &self.pg.finish_with_message(msg.clone());
//     }
// }
//
// impl ProgressFactory<AppProgress> for AppProgressFactory {
//     fn start(&self, msg: &String) -> AppProgress {
//         let pg = create_progress_bar();
//         pg.set_message(msg.clone());
//         AppProgress {
//             pg
//         }
//     }
// }
