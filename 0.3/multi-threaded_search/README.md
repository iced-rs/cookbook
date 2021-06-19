# Multi-Threaded Search

> It is recommended to run this example using a release build `cargo run --release`

<div align="center">
    <img src="https://github.com/GCI-Global/bathtub/blob/main/0.3/multi-threaded_search/img/example.gif?raw=true">
</div>

The main thing this example shows is how to use Iced with multi-threaded Rust to create blazingly fast software with ease. This example is by no means very optimized. This example is mostly an example of what a mild-ly experienced developer can do in a short timespan, because that is how it was built :)

## What this does
Very fast file searching. This example searches thousands of files extremely quickly by making each file search its own thread, this makes the search ~15 times faster than what a similarly skilled developer could do in Python for example.

## How it works
1. After each key press 15 threads (the maximum number of files that can be opened by one application in windows) are created
```rust
	Command::batch((0..min(15, state.unsearched_files.len())).into_iter().fold(
	Vec::with_capacity(15),
	|mut v, _i| {
		v.push(Command::perform(
		search_files(
			state.search_bars.iter().fold(
			Vec::with_capacity(state.search_bars.len()),
			|mut v, bar| {
				v.push(bar.value.to_lowercase().clone());
				v
			},
			),
			state.unsearched_files.remove(0),
		),
		Message::AddLog,
		));
		v
	},
	))
```
2. Each thread is checked if it contains 1 or more strings within it. If yes, the file is returned as a viewable Iced object
```rust
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
```
3. The returned file is added to the Iced state and a new search thread is created if neither all files have been searched or the maximum returned value is reached
```rust
	Message::AddLog((vals, log)) => {
	if state.logs.len() <= LOG_MAX
		&& vals
		.iter()
		.zip(state.search_bars.iter().fold(
			Vec::with_capacity(state.search_bars.len()),
			|mut v, bar| {
			v.push(&bar.value);
			v
			},
		))
		.all(|(a, b)| a == &b.to_lowercase())
	{
		if let Some(log) = log {
		state.logs.push(log);
		}
		if state.unsearched_files.len() > 0 {
		Command::perform(
			search_files(vals, state.unsearched_files.remove(0)),
			Message::AddLog,
		)
		} else {
		if let Some(start_time) = state.search_start {
		Command::perform(calc_speed(0, start_time), Message::GotSpeed)
		} else {Command::none()}
		}
	} else {
		let length = state.unsearched_files.len();
		state.unsearched_files.clear();
		if let Some(start_time) = state.search_start {
		Command::perform(calc_speed(length, start_time), Message::GotSpeed)
		} else {Command::none()}
	}
	}
```

## That's it!!
It may seem like a lot, but acheiving this level of speed in any other language would be a LOT more work. The bottle neck is not the code, the reported speed from this example is likely limited by your data storage medium!
This concept applies to way more than just searching log files, it could be used for searching websites databases, network scans, etc, mostly with only a few changes to step 2 above!