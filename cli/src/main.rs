use clap::{Parser, ValueEnum};
use std::collections::HashSet;
use std::sync::Arc;
use crossterm::execute;
use inquire::formatter::MultiOptionFormatter;
use inquire::list_option::ListOption;
use inquire::validator::Validation;
use inquire::MultiSelect;
use tabled::Table;
use tokio::task;
use indicatif::{ProgressBar, ProgressStyle};
use notify_rust::Notification;
use cleaner::clear_data;
use database::{get_pcbooster_version};
use database::structures::{CleanerData, CleanerResult, Cleared};
use database::utils::get_file_size_string;

#[cfg(windows)]
use std::io::stdin;
use std::io::stdout;

async fn work(disabled_programs: Vec<&str>, categories: Vec<String>, database: &Vec<CleanerData>) {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {prefix:.bold.dim} {spinner:.green}\n[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{msg}]",
    )
        .unwrap()
        .progress_chars("##-")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let mut bytes_cleared = 0;
    let mut removed_files = 0;
    let mut removed_directories = 0;
    let mut cleared_programs: Vec<Cleared> = Vec::new();

    let pb = ProgressBar::new(0);
    pb.set_style(sty.clone());
    pb.set_prefix("Clearing");

    let has_last_activity = categories.contains(&"LastActivity".to_string());

    let mut tasks = Vec::new();

    if has_last_activity {
        let progress_bar = Arc::new(pb.clone());
        let task = task::spawn(async move {
            progress_bar.set_message("LastActivity");
            #[cfg(windows)]
            database::registry_database::clear_last_activity();
            progress_bar.inc(1);
            CleanerResult {
                files: 0,
                folders: 0,
                bytes: 0,
                working: false,
                path: String::new(),
                program: String::new(),
                category: String::new(),
            }
        });
        tasks.push(task);
    }

    for data in database
        .iter()
        .filter(|data| categories.contains(&data.category.to_lowercase()))
    {
        if disabled_programs.contains(&data.program.to_lowercase().as_str()) {
            continue;
        }

        let data = Arc::new(data.clone());
        let progress_bar = Arc::new(pb.clone());
        let task = task::spawn(async move {
            progress_bar.set_message(data.path.clone());
            let result = clear_data(&data);
            progress_bar.inc(1);
            result
        });
        tasks.push(task);
    }

    pb.set_length(tasks.len() as u64);

    for task in tasks {
        match task.await {
            Ok(result) => {
                removed_files += result.files;
                removed_directories += result.folders;
                bytes_cleared += result.bytes;
                if result.working {
                    if let Some(cleared) = cleared_programs.iter_mut().find(|c| c.program == result.program) {
                        cleared.removed_bytes += result.bytes as u64;
                        cleared.removed_files += result.files as u64;
                        cleared.removed_directories += result.folders as u64;
                        if !cleared.affected_categories.contains(&result.category) {
                            cleared.affected_categories.push(result.category.clone());
                        }
                    } else {
                        let cleared = Cleared {
                            program: result.program.clone(),
                            removed_bytes: result.bytes as u64,
                            removed_files: result.files as u64,
                            removed_directories: result.folders as u64,
                            affected_categories: vec![result.category.clone()],
                        };
                        cleared_programs.push(cleared);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error waiting for task completion: {:?}", e);
            }
        }
    }

    pb.set_message("done");
    pb.finish();

    println!("Cleared result:");
    let table = Table::new(cleared_programs).to_string();
    println!("{}", table);
    println!("Removed size: {}", get_file_size_string(bytes_cleared));
    println!("Removed files: {}", removed_files);
    println!("Removed directories: {}", removed_directories);

    if let Err(e) = Notification::new()
        .summary("Cross Cleaner CLI")
        .body(&format!(
            "Removed: {}\nFiles: {}",
            get_file_size_string(bytes_cleared),
            removed_files
        ))
        .show()
    {
        eprintln!("Failed to show notification: {:?}", e);
    }
}

#[derive(Parser, Debug)]
#[command(1.0.0, None, long_about = None)]
struct Args {
    /// Specify categories to clear (comma-separated)
    #[arg(long, value_name = "CATEGORIES")]
    clear: Option<String>,

    /// Specify programs to disable (comma-separated)
    #[arg(long, value_name = "PROGRAMS")]
    disabled: Option<String>,
}


#[tokio::main]
async fn main() {
    execute!(
        stdout(),
        crossterm::terminal::SetTitle(format!("Cross Cleaner CLI v{}", get_pcbooster_version()))
    )
        .unwrap();

    let database: &Vec<CleanerData> = database::cleaner_database::get_database();

    let mut options: HashSet<String> = HashSet::new();
    let mut programs: HashSet<String> = HashSet::new();

    for data in database.to_vec() {
        let program = data.program.clone();
        options.insert(String::from(data.category.clone()));
        programs.insert(program);
    }

    println!(
        "DataBase programs: {}, DataBase paths: {}",
        programs.len(),
        database.len()
    );

    let validator = |a: &[ListOption<&&str>]| {
        if a.is_empty() {
            Ok(Validation::Invalid("No category is selected!".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let args = Args::parse();

    let clear_categories: HashSet<String> = args
        .clear
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_lowercase()) // Приводим к нижнему регистру
                .collect()
        })
        .unwrap_or_default();

    let disabled_programs: HashSet<String> = args
        .disabled
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_lowercase()) // Приводим к нижнему регистру
                .collect()
        })
        .unwrap_or_default();


    if clear_categories.is_empty() && disabled_programs.is_empty() {
        let formatter_categories: MultiOptionFormatter<'_, &str> =
            &|a| format!("{} selected categories", a.len());

        let options_str: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
        let ans_categories = MultiSelect::new(
            "Select the clearing categories:",
            options_str,
        )
            .with_validator(validator)
            .with_formatter(formatter_categories)
            .prompt();

        if let Ok(ans_categories) = ans_categories {
            let ans_categories: Vec<String> = ans_categories.into_iter().map(|s| s.to_string()).collect();

            let programs2: Vec<&str> = database
                .iter()
                .filter(|data| ans_categories.contains(&data.category))
                .map(|data| data.program.as_str())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let ans_programs = MultiSelect::new(
                "Select the disabled programs for clearing:",
                programs2,
            )
                .with_formatter(formatter_categories)
                .prompt();

            if let Ok(ans_programs) = ans_programs {
                work(
                    ans_programs,
                    ans_categories,
                    &database
                ).await;
            }
        }
    } else {
        let ans_categories: Vec<String> = clear_categories.into_iter().collect();
        let ans_programs: Vec<String> = disabled_programs.into_iter().collect();

        work(
            ans_programs.iter().map(|s| s.as_str()).collect(),
            ans_categories,
            &database,
        ).await;
    }

    #[cfg(windows)]
    {
        let mut s = String::new();
        let _ = stdin().read_line(&mut s);
    }
}