use std::fs;
use std::path::{Path};
use glob::glob;
use database::structures::{CleanerData, CleanerResult};

/// Рекурсивно удаляет директорию и обновляет счетчики в `cleaner_result`.
fn remove_directory_recursive(path: &Path, cleaner_result: &mut CleanerResult) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                remove_directory_recursive(&entry_path, cleaner_result)?;
                cleaner_result.folders += 1;
            } else {
                remove_file(&entry_path, cleaner_result)?;
            }
        }
        fs::remove_dir(path)?;
        cleaner_result.folders += 1;
    }
    Ok(())
}

/// Удаляет файл и обновляет счетчики в `cleaner_result`.
fn remove_file(path: &Path, cleaner_result: &mut CleanerResult) -> std::io::Result<()> {
    let metadata = fs::metadata(path)?;
    fs::remove_file(path)?;
    cleaner_result.bytes += metadata.len();
    cleaner_result.files += 1;
    Ok(())
}

/// Основная функция для очистки данных.
pub fn clear_data(data: &CleanerData) -> CleanerResult {
    let mut cleaner_result = CleanerResult {
        files: 0,
        folders: 0,
        bytes: 0,
        working: false,
        program: data.program.clone(),
        path: data.path.clone(),
    };

    // Используем glob для поиска файлов и директорий
    if let Ok(results) = glob(&data.path) {
        for result in results.flatten() {
            let path = result.as_path();
            let is_dir = path.is_dir();
            let is_file = path.is_file();

            // Удаление указанных файлов
            for file in &data.files_to_remove {
                let file_path = path.join(file);
                if file_path.exists() && file_path.is_file() {
                    if remove_file(&file_path, &mut cleaner_result).is_ok() {
                        cleaner_result.working = true;
                    }
                }
            }

            // Удаление указанных директорий
            for dir in &data.directories_to_remove {
                let dir_path = path.join(dir);
                if dir_path.exists() && dir_path.is_dir() {
                    if remove_directory_recursive(&dir_path, &mut cleaner_result).is_ok() {
                        cleaner_result.working = true;
                    }
                }
            }

            // Удаление всех файлов и директорий, если требуется
            if data.remove_all_in_dir && is_dir {
                if remove_directory_recursive(path, &mut cleaner_result).is_ok() {
                    cleaner_result.working = true;
                }
            }

            // Удаление файлов, если требуется
            if data.remove_files && is_file {
                if remove_file(path, &mut cleaner_result).is_ok() {
                    cleaner_result.working = true;
                }
            }

            // Удаление директорий, если требуется
            if data.remove_directories && is_dir {
                if remove_directory_recursive(path, &mut cleaner_result).is_ok() {
                    cleaner_result.working = true;
                }
            }

            // Удаление директории после очистки, если требуется
            if data.remove_directory_after_clean && is_dir {
                if fs::remove_dir_all(path).is_ok() {
                    cleaner_result.folders += 1;
                    cleaner_result.working = true;
                }
            }
        }
    }

    cleaner_result
}