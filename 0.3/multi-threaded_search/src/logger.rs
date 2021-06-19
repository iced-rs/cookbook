use std::path::Path;
use std::fs;

use crate::{Log, LOGS};

pub async fn search_files<'a>(
    vals: Vec<String>,
    file_name: String,
) -> (Vec<String>, Option<Log>) {
    let test_string = fs::read_to_string(Path::new(&format!("{}/{}", LOGS, file_name)))
        .unwrap()
        .to_lowercase();
    if vals.iter().all(|val| test_string.contains(val)) {
        // notice how we also create the log file in this separate thread?
        // It is not a huge improvement, because the `Log` type is very simple
        // but this is more performant than say sending back the file name and making
        // the main thread then create the `Log`
        (vals, Some(Log::new(file_name)))
    } else {
        (vals, None)
    }
}