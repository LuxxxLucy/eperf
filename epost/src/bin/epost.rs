// Epost: post-hoc egg analysis
// ========================================
//
// Author: Jialin Lu
// Contact: luxxxlucy@gmail.com
//
// Now supports:
//  1. Converts 'egg' library traces to a Linux perf-like script format,
//     facilitating visualization and analysis in profiling tools.

use epost::prelude::*;

fn main() -> Result<()> {
    let cmd = setup_clap_cli();
    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("script", matches)) => {
            let input_path = get_path_from_clap_matches(matches, "input")?;
            // todo: move to use the input file name as the default output name if not specified.
            let output_path = get_path_from_clap_matches(matches, "output_path")?;
            transform_traces_to_perf_script(&input_path, &output_path)?;
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };

    Ok(())
}

// Retrieves a string from the command line arguments
fn get_path_from_clap_matches(
    matches: &clap::ArgMatches,
    name: &str,
) -> Result<std::path::PathBuf> {
    matches
        .get_one::<std::path::PathBuf>(name)
        .cloned()
        .ok_or_else(|| Error::Generic(std::format!("{} not specified in the arguments", name)))
}

// Sets up the command-line interface using clap
fn setup_clap_cli() -> clap::Command {
    clap::Command::new("epost")
        .bin_name("epost")
        .about("EPOST: post-hoc analysis for equality saturation")
        .subcommand_required(true)
        .subcommand(
            clap::command!("script").args(&[
                clap::arg!(-o --output_path <PATH> "output path")
                    .value_parser(clap::value_parser!(std::path::PathBuf)),
                clap::arg!([input] "input path")
                    .value_parser(clap::value_parser!(std::path::PathBuf)),
            ]),
        )
}

/// Processes log data from the input path and converts it into a Linux `perf`-like script,
/// which is then written to the output path.
///
/// # Arguments
/// * `input_path` - Path to the input file containing log data.
/// * `output_path` - Path where the output script will be written.
///
/// # Returns
/// Returns a `Result` of `Ok` or `Error`.
fn transform_traces_to_perf_script(
    input_path: &std::path::PathBuf,
    output_path: &std::path::PathBuf,
) -> Result<()> {
    let lines = read_lines(input_path)?;
    let log_records = parse_trace_lines(lines)?;
    debug_print_log_records(&log_records);

    let script_lines = generate_perf_style_script(&log_records)?;
    write_lines(&script_lines, output_path)
}

fn debug_print_log_records(log_records: &[LogRecord]) {
    // Print log records for debugging or verification
    for (idx, log) in log_records.iter().enumerate() {
        println!("#{}: {:?}", idx, log);
    }
}

/// Reads all lines from a file at the given path.
///
/// # Arguments
/// * `path` - Path to the file to be read.
///
/// # Returns
/// Returns a `Result` containing either a `Vec<String>` of lines or an `Error`.
fn read_lines(path: &std::path::PathBuf) -> Result<Vec<String>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // reader.lines().collect::<Result<Vec<_>>>().map_err(Error::from)
    reader
        .lines()
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()
        .map_err(Error::from)
}

/// Writes lines to a file at the given path.
///
/// # Arguments
/// * `lines` - Lines to be written to the file.
/// * `path` - Path to the file where lines will be written.
///
/// # Returns
/// Returns a `Result` indicating the success or failure of the write operation.
fn write_lines(lines: &[String], path: &std::path::PathBuf) -> Result<()> {
    use std::fs::File;
    use std::io::{BufWriter, Write};

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

// Key-value pair structure used in log records
#[derive(Debug)]
struct KeyValuePair {
    key: String,
    value: String,
}

// Enum representing different types of log records
#[derive(Debug)]
enum LogRecordType {
    Start,
    End,
    Simple,
}

// Structure representing a single log record
#[derive(Debug)]
struct LogRecord {
    time: String,
    record_type: LogRecordType,
    function_name: String,
    arguments: Vec<KeyValuePair>,
}

// Parses log lines into log records
fn parse_trace_lines(lines: Vec<String>) -> Result<Vec<LogRecord>> {
    let mut lines = lines;
    lines.retain(|line| line.starts_with("EPOST_LOG"));

    let mut log_records: Vec<LogRecord> = Vec::new();
    let mut stack: Vec<String> = Vec::new();

    for line in &lines {
        let parts: Vec<&str> = line.splitn(2, ':').collect();

        // parsing prefix
        let prefix: Vec<&str> = parts[0].split_whitespace().collect();
        let epost_signature = prefix[0];
        if epost_signature != "EPOST_LOG" {
            return Err(Error::Generic(std::format!(
                "EPOST_LOG signature is wrong in line \"{}\"",
                line
            )));
        }
        let time = prefix[1].trim_matches('[').trim_matches(']');

        let parts: Vec<&str> = parts[1].split_whitespace().collect();
        let action = parts[parts.len() - 1];

        let function_and_args = match action {
            "START" | "END" => parts[0..parts.len() - 1].join(" "),
            _ => parts[0..].join(" "),
        };

        let function_args: Vec<&str> = function_and_args.split('(').collect();

        let function_name = match function_args.len() {
            2 => function_args[0].trim(),
            1 => function_and_args.trim(), // only function name with not arguments
            _ => {
                return Err(Error::Generic(std::format!(
                    "Incorrect format of function_and_args \"{:?}\" the line is\"{}\"",
                    function_args,
                    line
                )))
            } // Incorrect format for function and arguments
        };

        let mut arguments = Vec::new();
        if function_args.len() == 2 {
            let args_str = function_args[1].trim_matches(')').trim();
            for arg in args_str.split(", ") {
                let kv: Vec<&str> = arg.split('=').collect();
                if kv.len() == 2 {
                    arguments.push(KeyValuePair {
                        key: kv[0].to_string(),
                        value: kv[1].to_string(),
                    });
                }
            }
        }

        log_records.push(LogRecord {
            function_name: function_name.to_string(),
            time: time.to_string(),
            arguments,
            record_type: match action {
                "START" => LogRecordType::Start,
                "END" => LogRecordType::End,
                _ => LogRecordType::Simple,
            },
        });

        match action {
            "START" => {
                stack.push(function_name.to_string());
            }
            "END" => {
                if let Some(last) = stack.pop() {
                    if last != function_name {
                        return Err(Error::Generic(std::format!(
                            "Mismatch found: Expected '{}' but found '{}'",
                            function_name,
                            last
                        )));
                    }
                } else {
                    return Err(Error::Generic("Pop from empty stack!".to_string()));
                }
            }
            _ => {}
        }
    }

    if !stack.is_empty() {
        return Err(Error::Generic(
            "Unfinished actions remain in the stack!".to_string()
                ));
    }

    Ok(log_records)
}

// Formats log records into perf-style script
fn print_perf_style_stack(stack: &[String], time: &str, result_lines: &mut Vec<String>) {
    use std::fmt::Write;

    let mut my_string = String::new();
    let program_name: &str = "egg-run-program";
    let process: &str = "1";
    let post_str: &str = "1 cycles";

    let header = std::format!("{program_name} {process}   {time}:   {post_str}:");
    let f_counter = 1234;
    for (index, item) in stack.iter().rev().enumerate() {
        if index == 0 {
            writeln!(my_string, "{}", header).unwrap();
        }
        let source_name: &str = "egg-func-lib";
        writeln!(my_string, "\t{} {} ([{}])", f_counter, item, source_name).unwrap();
    }
    writeln!(my_string).unwrap();
    result_lines.push(my_string);
}

fn format_log_record(record: &LogRecord) -> String {
    let args = record
        .arguments
        .iter()
        .map(|arg| format!("{}={}", arg.key, arg.value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({})", record.function_name, args)
}

fn generate_perf_style_script(log_records: &[LogRecord]) -> Result<Vec<String>> {
    let mut stack: Vec<String> = Vec::new();
    let mut result_lines: Vec<String> = Vec::new();

    for log_record in log_records.iter() {
        match log_record.record_type {
            LogRecordType::Start => {
                stack.push(format_log_record(log_record));
                print_perf_style_stack(&stack, &log_record.time, &mut result_lines);
            }
            LogRecordType::End => {
                let Some(_last) = stack.pop() else {
                    return Err(Error::Generic("Pop from empty stack!".to_string()));
                };
                print_perf_style_stack(&stack, &log_record.time, &mut result_lines);
            }
            _ => {
                stack.push(format_log_record(log_record));
                print_perf_style_stack(&stack, &log_record.time, &mut result_lines);
                stack.pop().unwrap();
            }
        }
    }

    Ok(result_lines)
}
