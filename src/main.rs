use std::ffi::{CString, OsStr};
use std::fmt::{format, Debug};
use std::fs;
use std::io::stdin;
use std::path::Path;
use std::sync::Arc;
use crossterm::execute;
use glob::{glob, GlobResult, Paths, PatternError};
use inquire::formatter::MultiOptionFormatter;
use inquire::list_option::ListOption;
use inquire::MultiSelect;
use inquire::validator::Validation;
use tabled::{Table, Tabled};
use tokio::task;

#[derive(PartialEq, Tabled)]
struct Cleared {
    Program: String,

}
impl PartialEq<Option<Cleared>> for &Cleared {
    fn eq(&self, other: &Option<Cleared>) -> bool {
        match other {
            Some(other) => return other.Program.eq(&*self.Program),
            None => return false,
        }
    }
}
#[derive(Clone)]
struct CleanerData {
    pub path: String,
    pub category: String,
    pub program: String,

    pub files_to_remove: Vec<String>,
    pub folders_to_remove: Vec<String>,
    pub directories_to_remove: Vec<String>,

    pub remove_all_in_dir: bool,
    pub remove_directory_after_clean: bool,
    pub remove_directories: bool,
    pub remove_files: bool
}
struct CleanerResult {
    pub files: u64,
    pub folders: u64,
    pub bytes: u64,
    pub working: bool,
    pub program: String
}
fn get_steam_directory() {

}

fn clear_category(data: &CleanerData) -> CleanerResult{
    let mut cleaner_result: CleanerResult = CleanerResult { files: 0, folders: 0, bytes: 0, working: false, program: "".parse().unwrap() };
    let results: Result<Paths, PatternError> = glob(&*data.path);
    cleaner_result.program = (&*data.program).parse().unwrap();
    match results {
        Ok(results) => {
            for result in results {
                match result {
                    Ok(result) => {
                        let is_dir: bool = result.is_dir();
                        let is_file: bool = result.is_file();
                        let path: &str = result.as_path().to_str().unwrap();
                        let name: Option<&str> = result.file_name().unwrap().to_str();
                        let mut lenght = 0;
                        match result.metadata() {
                            Ok(res) => { lenght += res.len(); }
                            Err(_) => {}
                        }
                        //println!("Found: {}", path);
                        for file in &data.files_to_remove {
                            let file_path = path.to_owned() + "\\" + &*file;
                            match fs::remove_file(file_path) {
                                Ok(_) => {
                                    cleaner_result.files += 1;
                                    cleaner_result.bytes += lenght;
                                    cleaner_result.working = true;
                                    //println!("Removed file: {}", name.unwrap());
                                }
                                Err(_) => {}
                            }
                        }
                        for directory in &data.directories_to_remove {
                            let file_path = path.to_owned() + "\\" + &*directory;
                            let metadata = fs::metadata(file_path.clone());
                            match metadata {
                                Ok(res) => { lenght += res.len(); }
                                Err(_) => {}
                            }
                            match fs::remove_dir_all(file_path) {
                                Ok(_) => {
                                    cleaner_result.folders += 1;
                                    cleaner_result.working = true;
                                    //println!("Removed file: {}", name.unwrap());
                                }
                                Err(_) => {}
                            }
                        }

                        for dir in &data.directories_to_remove {
                            let dir_path = path.to_owned() + "\\" + &*dir;
                            let metadata = fs::metadata(dir_path.clone());
                            match metadata {
                                Ok(res) => { lenght += res.len(); }
                                Err(_) => {}
                            }
                            match fs::remove_dir(dir_path) {
                                Ok(_) => {
                                    cleaner_result.folders += 1;
                                    cleaner_result.bytes += lenght;
                                    cleaner_result.working = true;
                                    //println!("Removed directory: {}", name.unwrap());
                                }
                                Err(_) => {}
                            }
                        }

                        //println!("Found: {}", path);
                        if data.remove_files && is_file {
                            match fs::remove_file(path) {
                                Ok(_) => {
                                    cleaner_result.files += 1;
                                    cleaner_result.bytes += lenght;
                                    cleaner_result.working = true;
                                    //println!("Removed file: {}", name.unwrap());
                                }
                                Err(_) => {}
                            }
                        }
                        if data.remove_directories && is_dir {
                            match fs::remove_dir_all(path) {
                                Ok(_) => {
                                    cleaner_result.folders += 1;
                                    cleaner_result.bytes += lenght;
                                    cleaner_result.working = true;
                                    //println!("Removed directory: {}", name.unwrap());
                                }
                                Err(_) => {}
                            }
                        }
                        if data.remove_all_in_dir {
                            let results: Result<Paths, PatternError> = glob(&*(path.to_owned() + "\\*"));
                            let mut files = 0;
                            let mut dirs = 0;
                            match results {
                                Ok(results) => {
                                    for result in results {
                                        match result {
                                            Ok(result) => {
                                                if result.is_file() {
                                                    files += 1;
                                                }
                                                if result.is_dir() {
                                                    dirs += 1;
                                                }
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    match fs::remove_dir_all(path) {
                                        Ok(_) => {
                                            cleaner_result.files += files;
                                            cleaner_result.folders += dirs;
                                            cleaner_result.bytes += lenght;
                                            cleaner_result.working = true;
                                        }
                                        Err(_) => {}
                                    }
                                }
                                Err(_) => {}
                            }

                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }
    return cleaner_result;
}

fn get_file_size_string(size: u64) -> String {
    if size <= 0 {
        return "0 B".to_string();
    }

    let units = ["B", "KB", "MB", "GB", "TB"];
    let digit_groups = ((size as f64).log(1024.0)).floor() as usize;

    let size_in_units = size as f64 / 1024_f64.powi(digit_groups as i32);
    format!("{:.1} {}", size_in_units, units[digit_groups])
}


#[tokio::main]
async fn main() {
    execute!(
        std::io::stdout(),
        crossterm::terminal::SetTitle("WinBooster CLI v1.0.6")
    );

    let username = &*whoami::username();
    let mut database: Vec<CleanerData> = Vec::new();

    let mut options: Vec<&str> = vec![];
    let mut programs: Vec<&str> = vec![];
    //<editor-fold desc="Windows">
    let c_windows_debug_wia = CleanerData {
        path: "C:\\Windows\\debug\\WIA\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec!["wiatrace.log".parse().unwrap()],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_debug_wia);
    let c_windows_prefetch = CleanerData {
        path: "C:\\Windows\\Prefetch\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_prefetch);
    let c_windows_dumps = CleanerData {
        path: "C:\\Windows\\Minidump\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_dumps);
    let c_windows_security_logs = CleanerData {
        path: "C:\\Windows\\security\\logs\\*.log".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(), remove_directories: false,
        remove_files: true, directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_security_logs);
    let c_windows_security_database_logs = CleanerData {
        path: "C:\\Windows\\security\\database\\*.log".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_security_database_logs);

    let c_temp = CleanerData {
        path: "C:\\Temp\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_temp);
    let c_windows_panther = CleanerData {
        path: "C:\\Windows\\Panther".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false, directories_to_remove: vec![],
        remove_all_in_dir: true,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_panther);
    let c_windows_temp = CleanerData {
        path: "C:\\Windows\\Temp\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_temp);
    let c_windows_logs = CleanerData {
        path: "C:\\Windows\\Logs\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_logs);
    let c_windows_logs_windows_update = CleanerData {
        path: "C:\\Windows\\Logs\\WindowsUpdate\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_windows_logs_windows_update);
    let c_users_appdata_local_temp = CleanerData {
        path: "C:\\Users\\{username}\\AppData\\Local\\Temp\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_local_temp);
    let c_programdata_usoshared_logs = CleanerData {
        path: "C:\\ProgramData\\USOShared\\Logs\\*".parse().unwrap(),
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_programdata_usoshared_logs);
    let c_users_appdata_local_connecteddiveces_platform = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\ConnectedDevicesPlatform\\*",
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "LastActivity".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_local_connecteddiveces_platform);
    let c_users_appdata_local_crash_dumps = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\CrashDumps\\*",
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_local_crash_dumps);
    let c_users_downloads = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\Downloads\\*",
        program: "Windows".parse().unwrap(),
        files_to_remove: vec![],
        category: "Downloads".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_downloads);
    //</editor-fold>
    //<editor-fold desc="NVIDIA Corporation">
    let c_program_files_nvidia_corporation = CleanerData { 
        path: "C:\\Program Files\\NVIDIA Corporation".parse().unwrap(), 
        program: "NVIDIA Corporation".parse().unwrap(), 
        files_to_remove: vec!["license.txt".parse().unwrap()], 
        category: "Logs".parse().unwrap(), 
        remove_directories: false, 
        remove_files: false, 
        directories_to_remove: vec![], 
        remove_all_in_dir: false, 
        remove_directory_after_clean: false, 
        folders_to_remove: vec![]
    };
    database.push(c_program_files_nvidia_corporation);
    let c_program_files_nvidia_corporation_nvsmi = CleanerData { 
        path: "C:\\Program Files\\NVIDIA Corporation\\NVSMI".parse().unwrap(),
        program: "NVIDIA Corporation".parse().unwrap(), 
        files_to_remove: vec!["nvidia-smi.1.pdf".parse().unwrap()], 
        category: "Logs".parse().unwrap(), 
        remove_directories: false, 
        remove_files: false, 
        directories_to_remove: vec![], 
        remove_all_in_dir: false, 
        remove_directory_after_clean: false, 
        folders_to_remove: vec![] 
    };
    database.push(c_program_files_nvidia_corporation_nvsmi);
    //</editor-fold>
    //<editor-fold desc="Java">
    let java_1 = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.jdks\\**",
        program: "Java".parse().unwrap(),
        files_to_remove: vec![
            "javafx-src.zip".parse().unwrap(),
            "src.zip".parse().unwrap()
        ],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(java_1);
    let java_2 = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.jdks\\**",
        program: "Java".parse().unwrap(),
        files_to_remove: vec![
            "NOTICE".parse().unwrap(),
            "COPYRIGHT".parse().unwrap(),
            "LICENSE".parse().unwrap(),
            "release".parse().unwrap(),
            "README".parse().unwrap(),
            "ADDITIONAL_LICENSE_INFO".parse().unwrap(),
            "ASSEMBLY_EXCEPTION".parse().unwrap(),
            "Welcome.html".parse().unwrap(),
            "THIRDPARTYLICENSEREADME-JAVAFX.txt".parse().unwrap(),
            "THIRDPARTYLICENSEREADME.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "DISCLAIMER".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(java_2);
    let java_5 = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.jdks\\**",
        program: "Java".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![
            "sample".parse().unwrap(),
            "demo".parse().unwrap()
        ],
    };
    database.push(java_5);
    let java_2 = CleanerData {
        path: "C:\\Program Files\\Java\\**".parse().unwrap(),
        program: "Java".parse().unwrap(),
        files_to_remove: vec![
            "NOTICE".parse().unwrap(),
            "COPYRIGHT".parse().unwrap(),
            "LICENSE".parse().unwrap(),
            "release".parse().unwrap(),
            "README".parse().unwrap(),
            "ADDITIONAL_LICENSE_INFO".parse().unwrap(),
            "ASSEMBLY_EXCEPTION".parse().unwrap(),
            "Welcome.html".parse().unwrap(),
            "THIRDPARTYLICENSEREADME-JAVAFX.txt".parse().unwrap(),
            "THIRDPARTYLICENSEREADME.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "DISCLAIMER".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(java_2);
    let java_3 = CleanerData {
        path: "C:\\Program Files\\Eclipse Adoptium\\**".parse().unwrap(),
        program: "Java".parse().unwrap(),
        files_to_remove: vec![
            "NOTICE".parse().unwrap(),
            "COPYRIGHT".parse().unwrap(),
            "LICENSE".parse().unwrap(),
            "release".parse().unwrap(),
            "README".parse().unwrap(),
            "ADDITIONAL_LICENSE_INFO".parse().unwrap(),
            "ASSEMBLY_EXCEPTION".parse().unwrap(),
            "Welcome.html".parse().unwrap(),
            "THIRDPARTYLICENSEREADME-JAVAFX.txt".parse().unwrap(),
            "THIRDPARTYLICENSEREADME.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "DISCLAIMER".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(java_3);
    let java_4 = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\.loliland\\java",
        program: "Java".parse().unwrap(),
        files_to_remove: vec![
            "NOTICE".parse().unwrap(),
            "COPYRIGHT".parse().unwrap(),
            "LICENSE".parse().unwrap(),
            "release".parse().unwrap(),
            "README".parse().unwrap(),
            "ADDITIONAL_LICENSE_INFO".parse().unwrap(),
            "ASSEMBLY_EXCEPTION".parse().unwrap(),
            "Welcome.html".parse().unwrap(),
            "THIRDPARTYLICENSEREADME-JAVAFX.txt".parse().unwrap(),
            "THIRDPARTYLICENSEREADME.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "DISCLAIMER".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(java_4);
    //</editor-fold>
    //<editor-fold desc="4uKey for Android">
    let c_program_files_x86_tenorshare_4ukey_for_android_logs = CleanerData {
        path: "C:\\Program Files (x86)\\Tenorshare\\4uKey for Android\\Logs\\*".parse().unwrap(),
        program: "4uKey for Android".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_tenorshare_4ukey_for_android_logs);
    let c_users_appdata_roaming_tsmonitor_4uker_for_android = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\TSMonitor\\4uKey for Android\\logs\\*",
        program: "4uKey for Android".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_tsmonitor_4uker_for_android);
    //</editor-fold>
    //<editor-fold desc="Postman">
    let c_users_appdata_roaming_postman_agent_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\PostmanAgent\\logs\\*.log",
        program: "Postman".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_postman_agent_logs);
    let c_users_appdata_local_postman_agent = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Local\\Postman-Agent\\*.log",
        program: "4uKey for Android".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_postman_agent);
    //</editor-fold>
    //<editor-fold desc="IDA Pro">
    let c_users_appdata_roaming_hex_rays_ida_pro = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Hex-Rays\\IDA Pro\\*.lst",
        program: "IDA Pro".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_hex_rays_ida_pro);

    //</editor-fold>
    //<editor-fold desc="Xamarin"">
    let c_users_appdata_local_xamarin_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\Xamarin\\Logs\\**\\*.log",
        program: "Xamarin".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_xamarin_logs);
    //</editor-fold>
    //<editor-fold desc="Windscribe"">
    let c_users_appdata_local_windscribe = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\Windscribe\\Windscribe2\\*.txt",
        program: "Windscribe".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_windscribe);
    //</editor-fold>
    //<editor-fold desc="GitHub Desktop"">
    let c_users_appdata_roaming_github_desktop = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\GitHub Desktop\\*.log",
        program: "GitHub Desktop".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_github_desktop);
    let c_users_appdata_roaming_github_desktop_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\GitHub Desktop\\logs\\*.log",
        program: "GitHub Desktop".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_github_desktop_logs);
    //</editor-fold>
    //<editor-fold desc="Panda Security"">
    let c_programdata_panda_security_pslogs = CleanerData {
        path: "C:\\ProgramData\\Panda Security\\PSLogs\\*.log".parse().unwrap(),
        program: "Panda Security".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_panda_security_pslogs);
    //</editor-fold>
    //<editor-fold desc="NetLimiter"">
    let c_programdata_panda_security_pslogs = CleanerData {
        path: "C:\\ProgramData\\Locktime\\NetLimiter\\**\\logs\\*.log".parse().unwrap(),
        program: "NetLimiter".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_panda_security_pslogs);
    //</editor-fold>
    //<editor-fold desc="MiniBin"">
    let c_program_files_x86_minibin = CleanerData {
        path: "C:\\Program Files (x86)\\MiniBin\\*.txt".parse().unwrap(),
        program: "MiniBin".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_minibin);
    //</editor-fold>
    //<editor-fold desc="Brave Browser"">
    let c_program_files_brave_software_brave_browser_application = CleanerData {
        path: "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\*.log".parse().unwrap(),
        program: "Brave Browser".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_brave_software_brave_browser_application);
    let c_users_appdata_local_brave_software_brave_browser_user_data_default = CleanerData {
        path: "C:\\Users\\".to_owned() + username+ "\\AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default",
        program: "Brave Browser".parse().unwrap(),
        files_to_remove: vec![
            "Favicons".parse().unwrap(),
            "Favicons-journal".parse().unwrap(),
            "History".parse().unwrap(),
            "History-journal".parse().unwrap(),
            "Visited Links".parse().unwrap()
        ],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_brave_software_brave_browser_user_data_default);
    let c_users_appdata_local_brave_software_brave_browser_user_data_default_dawn_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\DawnCache",
        program: "Brave Browser".parse().unwrap(),
        files_to_remove: vec![
            "data_0".parse().unwrap(),
            "data_1".parse().unwrap(),
            "data_2".parse().unwrap(),
            "data_3".parse().unwrap(),
            "index".parse().unwrap()
        ],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_brave_software_brave_browser_user_data_default_dawn_cache);
    let c_users_appdata_local_brave_software_brave_browser_user_data_default_gpu_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\GPUCache",
        program: "Brave Browser".parse().unwrap(),
        files_to_remove: vec![
            "data_0".parse().unwrap(),
            "data_1".parse().unwrap(),
            "data_2".parse().unwrap(),
            "data_3".parse().unwrap(),
            "index".parse().unwrap()
        ],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_brave_software_brave_browser_user_data_default_gpu_cache);
    //</editor-fold>
    //<editor-fold desc="Mem Reduct"">
    let c_program_files_brave_software_brave_browser_application = CleanerData {
        path: "C:\\Program Files\\Mem Reduct".parse().unwrap(),
        program: "Mem Reduct".parse().unwrap(),
        files_to_remove: vec![
            "History.txt".parse().unwrap(),
            "License.txt".parse().unwrap(),
            "Readme.txt".parse().unwrap(),
            "memreduct.exe.sig".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_brave_software_brave_browser_application);
    //</editor-fold>
    //<editor-fold desc="qBittorrent"">
    let c_program_files_qbittorent = CleanerData {
        path: "C:\\Program Files\\qBittorrent\\*.pdb".parse().unwrap(),
        program: "qBittorrent".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_qbittorent);
    let c_program_files_qbittorent_logs = CleanerData {
        path: "C:\\Program Files\\qBittorrent\\logs\\*.log".parse().unwrap(),
        program: "qBittorrent".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_qbittorent_logs);
    //</editor-fold>
    //<editor-fold desc="CCleaner"">
    let c_program_files_ccleaner_logs = CleanerData {
        path: "C:\\Program Files\\CCleaner\\LOG\\*".parse().unwrap(),
        program: "CCleaner".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_ccleaner_logs);
    //</editor-fold>
    //<editor-fold desc="IObit Malware Fighter"">
    let c_program_files_ccleaner_logs = CleanerData {
        path: "C:\\ProgramData\\IObit\\IObit Malware Fighter\\*.log".parse().unwrap(),
        program: "IObit Malware Fighter".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_ccleaner_logs);
    let c_program_data_iobit_iobit_malware_finghter_homepage_advisor = CleanerData {
        path: "C:\\ProgramData\\IObit\\IObit Malware Fighter\\Homepage Advisor\\*.log".parse().unwrap(),
        program: "IObit Malware Fighter".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_data_iobit_iobit_malware_finghter_homepage_advisor);
    //</editor-fold>
    //<editor-fold desc="IObit Driver Booster"">
    let c_users_appdata_roaming_iobit_driver_booster_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\IObit\\Driver Booster\\Logs\\*",
        program: "IObit Driver Booster".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_iobit_driver_booster_logs);
    let c_program_files_x86_iobit_driver_booster = CleanerData {
        path: "C:\\Program Files (x86)\\IObit\\Driver Booster\\*.log".parse().unwrap(),
        program: "IObit Driver Booster".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_iobit_driver_booster);
    let c_program_files_x86_iobit_driver_booster_1 = CleanerData {
        path: "C:\\Program Files (x86)\\IObit\\Driver Booster\\*.txt".parse().unwrap(),
        program: "IObit Driver Booster".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_iobit_driver_booster_1);
    //</editor-fold>
    //<editor-fold desc="Process Lasso"">
    let c_program_data_process_lasso_logs = CleanerData {
        path: "C:\\ProgramData\\ProcessLasso\\logs\\*".parse().unwrap(),
        program: "Process Lasso".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_data_process_lasso_logs);
    //</editor-fold>
    //<editor-fold desc="OBS Studio"">
    let c_program_files_obs_studio_bin_64bit = CleanerData {
        path: "C:\\Program Files\\obs-studio\\bin\\64bit\\*.log".parse().unwrap(),
        program: "OBS Studio".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_obs_studio_bin_64bit);
    let c_users_appdata_roaming_obs_studio_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\obs-studio\\logs\\*txt",
        program: "OBS Studio".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_obs_studio_logs);
    //</editor-fold>
    //<editor-fold desc="Unity Hub"">
    let c_program_files_unity_hub = CleanerData {
        path: "C:\\Program Files\\Unity Hub\\*.html".parse().unwrap(),
        program: "Unity Hub".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_unity_hub);
    //</editor-fold>
    //<editor-fold desc="KeePass 2""">
    let c_program_files_keepass_password_safe_2 = CleanerData {
        path: "C:\\Program Files\\KeePass Password Safe 2\\*.txt".parse().unwrap(),
        program: "KeePass 2".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_keepass_password_safe_2);
    //</editor-fold>
    //<editor-fold desc="1Password""">
    let c_users_appdata_local_1password_logs_setup = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\1Password\\logs\\setup\\*.log",
        program: "1Password".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_1password_logs_setup);
    //</editor-fold>
    //<editor-fold desc="LGHUB""">
    let c_program_files_lghub = CleanerData {
        path: "C:\\Program Files\\LGHUB".parse().unwrap(),
        program: "LGHUB".parse().unwrap(),
        files_to_remove: vec![
            "LICENSE".parse().unwrap(),
            "LICENSES.chromium.html".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_lghub);
    //</editor-fold>
    //<editor-fold desc="DeepL""">
    let c_users_appdata_local_deepl_se_logs = CleanerData {
        path: "C:\\Users\\{username}\\AppData\\Local\\DeepL_SE\\logs\\*".parse().unwrap(),
        program: "LGHUB".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_deepl_se_logs);
    let c_users_appdata_local_deepl_se_cache = CleanerData {
        path: "C:\\Users\\{username}\\AppData\\Local\\DeepL_SE\\cache\\*".parse().unwrap(),
        program: "LGHUB".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_deepl_se_cache);
    //</editor-fold>
    //<editor-fold desc="Microsoft Lobe""">
    let c_users_appdata_roaming_lobe_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Lobe\\logs\\*",
        program: "Microsoft Lobe".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_lobe_logs);
    //</editor-fold>
    //<editor-fold desc="Tonfotos Telegram Connector""">
    let c_users_pictures_tonfotos_telegram_connector = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\Pictures\\Tonfotos Telegram Connector\\*",
        program: "Tonfotos Telegram Connector".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_pictures_tonfotos_telegram_connector);
    //</editor-fold>
    //<editor-fold desc="DotNet""">
    let c_program_files_x86_dotnet = CleanerData {
        path: "C:\\Program Files (x86)\\dotnet\\*.txt".parse().unwrap(),
        program: "DotNet".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_dotnet);
    let c_users_dotnet_telemetry_storage_service = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.dotnet\\TelemetryStorageService\\*",
        program: "DotNet".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_dotnet_telemetry_storage_service);
    //</editor-fold>
    //<editor-fold desc="MCCreator""">
    let c_users_mccreator_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.mcreator\\logs\\*.log",
        program: "MCCreator".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_mccreator_logs);
    //</editor-fold>
    //<editor-fold desc="7-Zip""">
    let c_program_files_7_zip = CleanerData {
        path: "C:\\Program Files\\7-Zip\\*.txt".parse().unwrap(),
        program: "7-Zip".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_7_zip);
    //</editor-fold>
    //<editor-fold desc="Tribler""">
    let c_users_appdata_roaming_tribler = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\.Tribler\\*.log",
        program: "Tribler".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_tribler);
    //</editor-fold>
    //<editor-fold desc="I2P""">
    let c_users_appdata_local_i2peasy_addressbook = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\i2peasy\\addressbook",
        program: "I2P".parse().unwrap(),
        files_to_remove: vec![
            "log.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_i2peasy_addressbook);
    let c_users_appdata_local_i2peasy = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\i2peasy",
        program: "I2P".parse().unwrap(),
        files_to_remove: vec![
            "eventlog.txt".parse().unwrap(),
            "wrapper.log".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_i2peasy);
    let c_users_appdata_local_i2peasy_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\i2peasy\\logs\\*",
        program: "I2P".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_i2peasy_logs);
    let c_users_appdata_local_i2peasy_licenses = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\i2peasy\\licenses\\*",
        program: "I2P".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_i2peasy_licenses);
    //</editor-fold>
    //<editor-fold desc="BoxedAppPacker""">
    let c_program_filex_x86_boxedapppacker = CleanerData {
        path: "C:\\Program Files (x86)\\BoxedAppPacker".parse().unwrap(),
        program: "BoxedAppPacker".parse().unwrap(),
        files_to_remove: vec![
            "changelog.txt".parse().unwrap(),
            "HomePage.url".parse().unwrap(),
            "purchase.url".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_filex_x86_boxedapppacker);
    //</editor-fold>
    //<editor-fold desc="Enigma Virtual Box""">
    let c_program_files_enigma_virtual_box = CleanerData {
        path: "C:\\Program Files (x86)\\Enigma Virtual Box".parse().unwrap(),
        program: "Enigma Virtual Box".parse().unwrap(),
        files_to_remove: vec![
            "help.chm".parse().unwrap(),
            "History.txt".parse().unwrap(),
            "License.txt".parse().unwrap(),
            "site.url".parse().unwrap(),
            "forum.url".parse().unwrap(),
            "support.url".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_enigma_virtual_box);
    //</editor-fold>
    //<editor-fold desc="GnuPG""">
    let c_program_files_gnupg = CleanerData {
        path: "C:\\Program Files (x86)\\GnuPG".parse().unwrap(),
        program: "GnuPG".parse().unwrap(),
        files_to_remove: vec![
            "README.txt".parse().unwrap(),
            "VERSION".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_gnupg);
    //</editor-fold>
    //<editor-fold desc="Gpg4win""">
    let c_program_files_enigma_x86_gpg4win = CleanerData {
        path: "C:\\Program Files (x86)\\Gpg4win".parse().unwrap(),
        program: "Gpg4win".parse().unwrap(),
        files_to_remove: vec![
            "VERSION".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_enigma_x86_gpg4win);
    //</editor-fold>
    //<editor-fold desc="Inno Setup 6""">
    let c_program_files_enigma_x86_inno_setup_6 = CleanerData {
        path: "C:\\Program Files (x86)\\Inno Setup 6".parse().unwrap(),
        program: "Inno Setup 6".parse().unwrap(),
        files_to_remove: vec![
            "whatsnew.htm".parse().unwrap(),
            "isfaq.url".parse().unwrap(),
            "license.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_enigma_x86_inno_setup_6);
    let c_program_files_enigma_x86_inno_setup_6 = CleanerData {
        path: "C:\\Program Files (x86)\\Inno Setup 6\\Examples\\*.txt".parse().unwrap(),
        program: "Inno Setup 6".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_enigma_x86_inno_setup_6);
    //</editor-fold>
    //<editor-fold desc="VirtualBox""">
    let c_users_virtualbox_vms_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\VirtualBox VMs\\**\\Logs\\*.log",
        program: "VirtualBox".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_virtualbox_vms_logs);
    let c_users_virtualbox_vms = CleanerData {
        path: "C:\\Program Files\\Oracle\\VirtualBox\\*.rtf".parse().unwrap(),
        program: "VirtualBox".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_virtualbox_vms);
    let c_users_virtualbox_vms_doc = CleanerData {
        path: "C:\\Program Files\\Oracle\\VirtualBox\\doc\\*".parse().unwrap(),
        program: "VirtualBox".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_virtualbox_vms_doc);
    //</editor-fold>
    //<editor-fold desc="Recaf""">
    let c_users_appdata_roaming_recaf = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Recaf\\*.log",
        program: "Recaf".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_recaf);
    //</editor-fold>
    //<editor-fold desc="Process Hacker 2""">
    let c_program_files_process_hacker_2 = CleanerData {
        path: "C:\\Program Files\\Process Hacker 2".parse().unwrap(),
        program: "Process Hacker 2".parse().unwrap(),
        files_to_remove: vec![
            "CHANGELOG.txt".parse().unwrap(),
            "COPYRIGHT.txt".parse().unwrap(),
            "LICENSE.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "ProcessHacker.sig".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_process_hacker_2);
    //</editor-fold>
    //<editor-fold desc="Docker""">
    let c_programdata_dockerdesktop = CleanerData {
        path: "C:\\ProgramData\\DockerDesktop\\*.txt".parse().unwrap(),
        program: "Docker".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_dockerdesktop);
    let c_users_appdata_local_docker_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Local\\Docker\\log\\**\\*",
        program: "Docker".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_docker_logs);
    let c_users_appdata_local_docker = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\Docker\\*.txt",
        program: "Docker".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_docker);
    //</editor-fold>
    //<editor-fold desc="HiAlgo Boost""">
    let c_programdata_dockerdesktop = CleanerData {
        path: "C:\\Program Files (x86)\\HiAlgo\\Plugins\\BOOST".parse().unwrap(),
        program: "HiAlgo Boost".parse().unwrap(),
        files_to_remove: vec![
            "hialgo_eula.txt".parse().unwrap(),
            "Update Boost.log".parse().unwrap(),
            "UpdateListing.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_dockerdesktop);
    //</editor-fold>
    //<editor-fold desc="SoundWire Server""">
    let c_program_files_x86_soundwire_server = CleanerData {
        path: "C:\\Program Files (x86)\\SoundWire Server".parse().unwrap(),
        program: "SoundWire Server".parse().unwrap(),
        files_to_remove: vec![
            "license.txt".parse().unwrap(),
            "opus_license.txt".parse().unwrap(),
            "readerwriterqueue_license.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_soundwire_server);
    //</editor-fold>
    //<editor-fold desc="System Informer""">
    let c_program_files_systeminformer = CleanerData {
        path: "C:\\Program Files\\SystemInformer".parse().unwrap(),
        program: "SoundWire Server".parse().unwrap(),
        files_to_remove: vec![
            "LICENSE.txt".parse().unwrap(),
            "README.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_systeminformer);
    //</editor-fold>
    //<editor-fold desc="Sandboxie Plus""">
    let c_program_files_sandboxie_plus = CleanerData {
        path: "C:\\Program Files\\Sandboxie-Plus".parse().unwrap(),
        program: "SoundWire Server".parse().unwrap(),
        files_to_remove: vec![
            "Manifest0.txt".parse().unwrap(),
            "Manifest1.txt".parse().unwrap(),
            "Manifest2.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_sandboxie_plus);
    //</editor-fold>
    //<editor-fold desc="JetBrains""">
    let c_program_files_jetbrains_license = CleanerData {
        path: "C:\\Program Files\\JetBrains\\**\\license".parse().unwrap(),
        program: "JetBrains".parse().unwrap(),
        files_to_remove: vec![
            "javahelp_license.txt".parse().unwrap(),
            "javolution_license.txt".parse().unwrap(),
            "launcher-third-party-libraries.html".parse().unwrap(),
            "saxon-conditions.html".parse().unwrap(),
            "third-party-libraries.html".parse().unwrap(),
            "yourkit-license-redist.txt".parse().unwrap(),
            "remote-dev-server.html".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_jetbrains_license);
    let c_program_files_jetbrains = CleanerData {
        path: "C:\\Program Files\\JetBrains\\**".parse().unwrap(),
        program: "JetBrains".parse().unwrap(),
        files_to_remove: vec![
            "LICENSE.txt".parse().unwrap(),
            "NOTICE.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_jetbrains);
    //</editor-fold>
    //<editor-fold desc="AAF Optimus DCH Audio""">
    let c_program_files_afftweak = CleanerData {
        path: "C:\\Program Files\\AAFTweak".parse().unwrap(),
        program: "AAF Optimus DCH Audio".parse().unwrap(),
        files_to_remove: vec![
            "RT.pdb".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_afftweak);
    //</editor-fold>
    //<editor-fold desc="FL Studio""">
    let c_program_files_image_line = CleanerData {
        path: "C:\\Program Files\\Image-Line\\**".parse().unwrap(),
        program: "FL Studio".parse().unwrap(),
        files_to_remove: vec![
            "WhatsNew.rtf".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_image_line);
    let c_program_files_image_line_shared_start = CleanerData {
        path: "C:\\Program Files\\Image-Line\\Shared\\Start\\**".parse().unwrap(),
        program: "FL Studio".parse().unwrap(),
        files_to_remove: vec![
            "What's new.lnk".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_image_line_shared_start);
    //</editor-fold>
    //<editor-fold desc="ASIO4ALL v2""">
    let c_program_files_x86_asio4all = CleanerData {
        path: "C:\\Program Files (x86)\\ASIO4ALL v2".parse().unwrap(),
        program: "ASIO4ALL v2".parse().unwrap(),
        files_to_remove: vec![
            "ASIO4ALL Web Site.url".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_asio4all);
    //</editor-fold>
    //<editor-fold desc="VideoLAN""">
    let c_program_files_videolan_vlc = CleanerData {
        path: "C:\\Program Files\\VideoLAN\\VLC".parse().unwrap(),
        program: "ASIO4ALL v2".parse().unwrap(),
        files_to_remove: vec![
            "AUTHORS.txt".parse().unwrap(),
            "COPYING.txt".parse().unwrap(),
            "NEWS.txt".parse().unwrap(),
            "README.txt".parse().unwrap(),
            "THANKS.txt".parse().unwrap(),
            "VideoLAN Website.url".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_videolan_vlc);
    //</editor-fold>
    //<editor-fold desc="HandBrake""">
    let c_users_appdata_roaming_handbrake_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\HandBrake\\logs\\*.txt",
        program: "HandBrake".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_handbrake_logs);
    let c_users_appdata_roaming_handbrake_docs = CleanerData {
        path: "C:\\Program Files\\HandBrake\\doc\\*".parse().unwrap(),
        program: "HandBrake".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_handbrake_docs);
    //</editor-fold>
    //<editor-fold desc="Topaz Video AI"""">
    let c_programdata_topaz_labs_llc_topaz_video_ai = CleanerData {
        path: "C:\\ProgramData\\Topaz Labs LLC\\Topaz Video AI\\*.txt".parse().unwrap(),
        program: "Topaz Video AI".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_topaz_labs_llc_topaz_video_ai);
    //</editor-fold>
    //<editor-fold desc="AVCLabs Video Enhancer AI"""">
    let c_program_files_x86_avclabs_avclabs_video_encharcer_ai_1 = CleanerData {
        path: "C:\\Program Files (x86)\\AVCLabs\\AVCLabs Video Enhancer AI\\*.txt".parse().unwrap(),
        program: "AVCLabs Video Enhancer AI".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_avclabs_avclabs_video_encharcer_ai_1);
    let c_program_files_x86_avclabs_avclabs_video_encharcer_ai_2 = CleanerData {
        path: "C:\\Program Files (x86)\\AVCLabs\\AVCLabs Video Enhancer AI\\*.html".parse().unwrap(),
        program: "AVCLabs Video Enhancer AI".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_avclabs_avclabs_video_encharcer_ai_2);
    let c_program_files_x86_avclabs_avclabs_video_encharcer_ai_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\AVCLabs Video Enhancer AI\\logs\\*.log",
        program: "AVCLabs Video Enhancer AI".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_avclabs_avclabs_video_encharcer_ai_logs);
    //</editor-fold>
    //<editor-fold desc="iTop Screen Recorder"""">
    let c_users_appdata_roaming_itop_screen_recorder_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\iTop Screen Recorder\\Logs\\*.log",
        program: "iTop Screen Recorder".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_itop_screen_recorder_logs);
    //</editor-fold>
    //<editor-fold desc="Rave"""">
    let c_users_appdata_roaming_rave_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Rave\\logs\\*.log",
        program: "Rave".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_rave_logs);
    let c_users_appdata_roaming_rave_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Rave\\Cache\\*",
        program: "Rave".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_rave_cache);
    let c_users_appdata_roaming_rave_code_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Rave\\Code Cache\\*",
        program: "Rave".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_rave_code_cache);
    //</editor-fold>
    //<editor-fold desc="ImageGlass"""">
    let c_program_files_imageglass = CleanerData {
        path: "C:\\Program Files\\ImageGlass".parse().unwrap(),
        program: "ImageGlass".parse().unwrap(),
        files_to_remove: vec![
            "ReadMe.rtf".parse().unwrap(),
            "CliWrap.xml".parse().unwrap(),
            "DotNetZip.pdb".parse().unwrap(),
            "DotNetZip.xml".parse().unwrap(),
            "ImageGlass.ImageBox.xml".parse().unwrap(),
            "ImageGlass.ImageListView.xml".parse().unwrap(),
            "LICENSE".parse().unwrap(),
            "Magick.NET.Core.xml".parse().unwrap(),
            "Magick.NET.SystemDrawing.xml".parse().unwrap(),
            "Magick.NET-Q16-HDRI-OpenMP-x64.xml".parse().unwrap(),
            "Microsoft.Bcl.AsyncInterfaces.xml".parse().unwrap(),
            "System.Buffers.xml".parse().unwrap(),
            "System.Memory.xml".parse().unwrap(),
            "System.Numerics.Vectors.xml".parse().unwrap(),
            "System.Runtime.CompilerServices.Unsafe.xml".parse().unwrap(),
            "System.Text.Encodings.Web.xml".parse().unwrap(),
            "System.Text.Json.xml".parse().unwrap(),
            "System.Threading.Tasks.Extensions.xml".parse().unwrap(),
            "System.ValueTuple.xml".parse().unwrap(),"".parse().unwrap(),
            "ImageGlass.WebP.pdb".parse().unwrap(),
            "Visit ImageGlass website.url".parse().unwrap(),
            "default.jpg".parse().unwrap()

        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_imageglass);
    //</editor-fold>
    //<editor-fold desc="InkSpace"""">
    let c_program_files_inkscape = CleanerData {
        path: "C:\\Program Files\\Inkscape".parse().unwrap(),
        program: "InkSpace".parse().unwrap(),
        files_to_remove: vec![
            "NEWS.md".parse().unwrap(),
            "README.md".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_inkscape);
    let c_users_appdata_roaming_inkscape = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\inkscape\\*.log",
        program: "InkSpace".parse().unwrap(),
        files_to_remove: vec![ ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_inkscape);
    //</editor-fold>
    //<editor-fold desc="Magpie"""">
    let c_program_files_magpie_logs = CleanerData {
        path: "C:\\Program Files\\Magpie\\logs\\*.log".parse().unwrap(),
        program: "Magpie".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_magpie_logs);
    let c_program_files_magpie_logs = CleanerData {
        path: "C:\\Program Files\\Magpie\\cache\\*".parse().unwrap(),
        program: "Magpie".parse().unwrap(),
        files_to_remove: vec![ ],
        category: "Cache".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_magpie_logs);
    //</editor-fold>
    //<editor-fold desc="Notepad++"""">
    let c_users_appdata_roaming_notepad_plus_plus = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Notepad++\\*.log",
        program: "Notepad++".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_notepad_plus_plus);
    let c_program_files_notepad_plus_plus = CleanerData {
        path: "C:\\Program Files\\Notepad++\\*.log".parse().unwrap(),
        program: "Notepad++".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_notepad_plus_plus);
    let c_program_files_notepad_plus_plus_1 = CleanerData {
        path: "C:\\Program Files\\Notepad++\\*.txt".parse().unwrap(),
        program: "Notepad++".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_notepad_plus_plus_1);
    let c_program_files_notepad_plus_plus_2 = CleanerData {
        path: "C:\\Program Files\\Notepad++\\*LICENSE*".parse().unwrap(),
        program: "Notepad++".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_notepad_plus_plus_2);
    //</editor-fold>
    //<editor-fold desc="Sublime Text"""">
    let c_program_files_sublime_text = CleanerData {
        path: "C:\\Program Files\\Sublime Text\\*.txt".parse().unwrap(),
        program: "Sublime Text".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_sublime_text);
    //</editor-fold>
    //<editor-fold desc="LibreOffice"""">
    let c_program_files_libreoffice = CleanerData {
        path: "C:\\Program Files\\LibreOffice".parse().unwrap(),
        program: "LibreOffice".parse().unwrap(),
        files_to_remove: vec![
            "CREDITS.fodt".parse().unwrap(),
            "LICENSE.html".parse().unwrap(),
            "license.txt".parse().unwrap(),
            "NOTICE".parse().unwrap(),
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_libreoffice);
    let c_program_files_libreoffice_readmes = CleanerData {
        path: "C:\\Program Files\\LibreOffice\\readmes\\*".parse().unwrap(),
        program: "LibreOffice".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_libreoffice_readmes);
    //</editor-fold>
    //<editor-fold desc="Exodus Crypto Wallet"""">
    let c_users_appdata_local_exodus = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\exodus\\*.log",
        program: "Exodus Crypto Wallet".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_exodus);
    //</editor-fold>
    //<editor-fold desc="Wasabi Wallet"""">
    let c_users_appdata_roaming_walletwasabi_client = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\WalletWasabi\\Client\\*.txt",
        program: "Wasabi Wallet".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_walletwasabi_client);
    //</editor-fold>
    //<editor-fold desc="Bit Monero"""">
    let c_programdata_bitmonero = CleanerData {
        path: "C:\\ProgramData\\bitmonero\\*.log".parse().unwrap(),
        program: "Bit Monero".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_bitmonero);
    //</editor-fold>
    //<editor-fold desc="Memu"""">
    let c_users_memuhyperv = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.MemuHyperv\\*log*",
        program: "Memu".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_memuhyperv);
    //</editor-fold>
    //<editor-fold desc="Gameloop">
    let c_users_appdata_roaming_gametop_launcher = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\com.gametop.launcher\\logs\\*",
        program: "Gameloop".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_gametop_launcher);
    //</editor-fold>
    //<editor-fold desc="BlueStacks 5"""">
    let c_appdata_bluestacks_nxt_logs = CleanerData {
        path: "C:\\ProgramData\\BlueStacks_nxt\\Logs\\*.log".parse().unwrap(),
        program: "BlueStacks 5".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_appdata_bluestacks_nxt_logs);
    let c_users_pictures_bluestacks = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\Pictures\\BlueStacks\\*.png",
        program: "BlueStacks 5".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_pictures_bluestacks);
    //</editor-fold>
    //<editor-fold desc="GameGuard 5">
    let c_program_files_x86_gameguard_cache = CleanerData {
        path: "C:\\Program Files (x86)\\GameGuard\\cache\\*.cache".parse().unwrap(),
        program: "GameGuard".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_x86_gameguard_cache);
    //</editor-fold>
    //<editor-fold desc="FACEIT AC" 5">
    let c_program_files_faceit_ac_logs = CleanerData {
        path: "C:\\Program Files\\FACEIT AC\\logs\\*.log".parse().unwrap(),
        program: "FACEIT AC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_faceit_ac_logs);
    //</editor-fold>
    //<editor-fold desc="EasyAntiCheat">
    let c_program_files_faceit_ac_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\EasyAntiCheat\\*.log",
        program: "EasyAntiCheat".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_faceit_ac_logs);
    //</editor-fold>
    //<editor-fold desc="Cheat Engine">
    let c_program_files_faceit_ac_logs = CleanerData {
        path: "C:\\Program Files\\Cheat Engine 7.5\\*.txt".parse().unwrap(),
        program: "Cheat Engine".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_faceit_ac_logs);
    //</editor-fold>
    //<editor-fold desc="PowerToys">
    let krnl = CleanerData {
        path: "C:\\Program Files\\PowerToys".parse().unwrap(),
        program: "PowerToys".parse().unwrap(),
        files_to_remove: vec![
            "License.rtf".parse().unwrap(),
            "Notice.md".parse().unwrap()
        ],
        category: "Cheats".parse().unwrap(),
        remove_directories: true,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(krnl);
    //</editor-fold>

    //<editor-fold desc="Games">

    //<editor-fold desc="Minecraft Clients">

    //<editor-fold desc="Minecraft">
    let c_users_appdata_roaming_minecraft_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\.minecraft\\logs\\*",
        program: "Minecraft".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_minecraft_logs);
    let c_users_appdata_roaming_minecraft_saves = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\.minecraft\\saves\\*",
        program: "Minecraft".parse().unwrap(),
        files_to_remove: vec![],
        category: "Saves".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_minecraft_saves);
    //</editor-fold>
    //<editor-fold desc="Lunar Client">
    let c_users_appdata_lunarclient_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.lunarclient\\logs\\launcher\\*",
        program: "Minecraft".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_lunarclient_logs);
    let c_users_appdata_lunarclient_licenses = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\.lunarclient\\licenses\\*",
        program: "Minecraft".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_lunarclient_licenses);
    //</editor-fold>
    //<editor-fold desc="PrismLauncher Client">
    let c_users_appdata_roaming_prismlauncher = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\PrismLauncher\\*.log",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher);
    let c_users_appdata_roaming_prismlauncher_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\PrismLauncher\\logs\\*.log",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher_logs);
    let c_users_appdata_roaming_prismlauncher_instances_minecraft_crash_reports = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PrismLauncher\\instances\\**\\minecraft\\crash-reports\\*",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher_instances_minecraft_crash_reports);
    let c_users_appdata_roaming_prismlauncher_instances_minecraft_crash_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PrismLauncher\\instances\\**\\minecraft\\logs\\*",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher_instances_minecraft_crash_logs);
    let c_users_appdata_roaming_prismlauncher_instances_minecraft_screenshots = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PrismLauncher\\instances\\**\\minecraft\\screenshots\\*",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    let c_users_appdata_roaming_prismlauncher_instances_minecraft_saves = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PrismLauncher\\instances\\**\\minecraft\\saves\\*",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Saves".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher_instances_minecraft_saves);
    let c_users_appdata_roaming_prismlauncher_instances_minecraft_screenshots = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PrismLauncher\\cache\\*",
        program: "PrismLauncher".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_prismlauncher_instances_minecraft_screenshots);
    //</editor-fold>
    //<editor-fold desc="PolyMC Client">
    let c_users_appdata_roaming_polymc = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\PolyMC\\*.log",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc);
    let c_users_appdata_roaming_polymc_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\PolyMC\\logs\\*.log",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc_logs);
    let c_users_appdata_roaming_polymc_instances_minecraft_crash_reports = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PolyMC\\instances\\**\\minecraft\\crash-reports\\*",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc_instances_minecraft_crash_reports);
    let c_users_appdata_roaming_polymc_instances_minecraft_crash_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PolyMC\\instances\\**\\minecraft\\logs\\*",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc_instances_minecraft_crash_logs);
    let c_users_appdata_roaming_polymc_instances_minecraft_screenshots = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PolyMC\\instances\\**\\minecraft\\screenshots\\*",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    let c_users_appdata_roaming_polymc_instances_minecraft_saves = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PolyMC\\instances\\**\\minecraft\\saves\\*",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Saves".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc_instances_minecraft_saves);
    let c_users_appdata_roaming_polymc_instances_minecraft_screenshots = CleanerData {
        path: "C:\\Users\\".to_owned() + username +"\\AppData\\Roaming\\PolyMC\\cache\\*",
        program: "PolyMC".parse().unwrap(),
        files_to_remove: vec![],
        category: "Images".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![]
    };
    database.push(c_users_appdata_roaming_polymc_instances_minecraft_screenshots);
    //</editor-fold>

    //</editor-fold>

    //</editor-fold>
    //<editor-fold desc="Messangers""">
    //<editor-fold desc="Discord""">
    let c_users_appdata_local_discord = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\Discord\\*.log",
        program: "Discord".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_discord);
    //</editor-fold>
    //<editor-fold desc="Guilded""">
    let c_users_appdata_roaming_guilded = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Guilded\\*.log",
        program: "Guilded".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_guilded);
    //</editor-fold>
    //<editor-fold desc="Element""">
    let c_users_appdata_local_element_desktop = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\element-desktop\\*.log",
        program: "Element".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_element_desktop);
    //</editor-fold>
    //<editor-fold desc="Telegram""">
    let c_users_appdata_roaming_telefram_desktop_tdata = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\tdata",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![
            "key_datas".parse().unwrap()
        ],
        category: "Accounts".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop_tdata);
    let c_users_appdata_roaming_telefram_desktop = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\*.txt",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop);
    let c_users_appdata_roaming_telefram_desktop_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\*.log",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop_logs);
    let c_users_appdata_roaming_telefram_desktop_tdata_emoji_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\tdata\\emoji\\*cache_*",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop_tdata_emoji_cache);
    let c_users_appdata_roaming_telefram_desktop_tdata_user_data_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\tdata\\user_data\\cache\\**\\*",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop_tdata_user_data_cache);
    let c_users_appdata_roaming_telefram_desktop_tdata_user_data_media_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Telegram Desktop\\tdata\\user_data\\media_cache\\**\\*",
        program: "Telegram".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_telefram_desktop_tdata_user_data_media_cache);
    //</editor-fold>
    //<editor-fold desc="Signal""">
    let c_users_appdata_roaming_signal = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Signal\\logs\\*",
        program: "Signal".parse().unwrap(),
        files_to_remove: vec![ ],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_signal);
    let c_users_appdata_roaming_signal_update_cache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Signal\\update-cache\\*",
        program: "Signal".parse().unwrap(),
        files_to_remove: vec![ ],
        category: "Cache".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_roaming_signal_update_cache);
    //</editor-fold>
    //</editor-fold>
    //<editor-fold desc="VPN Clients""">
    //<editor-fold desc="Amnezia VPN""">
    let c_program_files_amnezia_vpn = CleanerData {
        path: "C:\\Program Files\\AmneziaVPN".parse().unwrap(),
        program: "Amnezia VPN".parse().unwrap(),
        files_to_remove: vec![
            "InstallationLog.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_amnezia_vpn);
    let c_program_files_amnezia_vpn_tap = CleanerData {
        path: "C:\\Program Files\\AmneziaVPN\\tap".parse().unwrap(),
        program: "Amnezia VPN".parse().unwrap(),
        files_to_remove: vec![
            "license.txt".parse().unwrap()
        ],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_amnezia_vpn_tap);
    //</editor-fold>
    //<editor-fold desc="Radmin VPN""">
    let c_program_filex_x86_radmin_vpn_chatlogs = CleanerData {
        path: "C:\\Program Files (x86)\\Radmin VPN\\CHATLOGS\\*".parse().unwrap(),
        program: "Radmin VPN".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_filex_x86_radmin_vpn_chatlogs);
    let c_program_files_amnezia_vpn = CleanerData {
        path: "C:\\ProgramData\\Famatech\\Radmin VPN\\*.txt".parse().unwrap(),
        program: "Radmin VPN".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_program_files_amnezia_vpn);
    //</editor-fold>
    //<editor-fold desc="UrbanVPN""">
    let c_users_urbanvpm_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\UrbanVPN\\log\\*",
        program: "UrbanVPN".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_urbanvpm_logs);
    //</editor-fold>
    //<editor-fold desc="CloudFlare""">
    let c_users_urbanvpm_logs = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\Cloudflare\\*.log",
        program: "CloudFlare".parse().unwrap(),
        files_to_remove: vec![],
        category: "Logs".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_urbanvpm_logs);
    //</editor-fold>
    //<editor-fold desc="PlanetVPN""">
    let c_users_appdata_local_planetvpn_cache_qmlcache = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Local\\PlanetVPN\\cache\\qmlcache\\*",
        program: "PlanetVPN".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_users_appdata_local_planetvpn_cache_qmlcache);
    //</editor-fold>
    //<editor-fold desc="iTop VPN""">
    let c_programdata_itop_vpn = CleanerData {
        path: "C:\\ProgramData\\iTop VPN".parse().unwrap(),
        program: "iTop VPN".parse().unwrap(),
        files_to_remove: vec![
            "iTop_setup.log".parse().unwrap(),
            "Setup.log".parse().unwrap()
        ],
        category: "Cache".parse().unwrap(),
        remove_directories: false,
        remove_files: false,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: false,
        folders_to_remove: vec![],
    };
    database.push(c_programdata_itop_vpn);
    //</editor-fold>
    //</editor-fold>
    //<editor-fold desc="Images">

    //<editor-fold desc="ShareX">
    let sharex_1 = CleanerData { path: "C:\\Users\\".to_owned() + username + "\\Documents\\ShareX\\Screenshots\\**\\*.jpg", program: "ShareX".parse().unwrap(), files_to_remove: vec![], category: "Images".parse().unwrap(), remove_directories: false, remove_files: true, directories_to_remove: vec![], remove_all_in_dir: false, remove_directory_after_clean: false, folders_to_remove: vec![] };
    database.push(sharex_1);
    let sharex_2 = CleanerData { path: "C:\\Users\\".to_owned() + username + "\\Documents\\ShareX\\Screenshots\\**\\*.png", program: "ShareX".parse().unwrap(), files_to_remove: vec![], category: "Images".parse().unwrap(), remove_directories: false, remove_files: true, directories_to_remove: vec![], remove_all_in_dir: false, remove_directory_after_clean: false, folders_to_remove: vec![] };
    database.push(sharex_2);
    let sharex_3 = CleanerData { path: "C:\\Users\\".to_owned() + username + "\\Documents\\ShareX\\Logs\\*", program: "ShareX".parse().unwrap(), files_to_remove: vec![], category: "Logs".parse().unwrap(), remove_directories: false, remove_files: true, directories_to_remove: vec![], remove_all_in_dir: false, remove_directory_after_clean: false, folders_to_remove: vec![] };
    database.push(sharex_3);
    //</editor-fold>

    //</editor-fold>
    //<editor-fold desc="Cheats">

    //<editor-fold desc="Weave">
    let weave_1 = CleanerData {
        path: "C:\\Weave\\*".parse().unwrap(),
        program: "Weave".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cheats".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(weave_1);
    //</editor-fold>
    //<editor-fold desc="INTERIUM">
    let interium = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\INTERIUM\\*",
        program: "INTERIUM".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cheats".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(interium);
    //</editor-fold>
    //<editor-fold desc="Krnl">
    let krnl = CleanerData {
        path: "C:\\Users\\".to_owned() + username + "\\AppData\\Roaming\\Krnl\\*",
        program: "Krnl".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cheats".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(krnl);
    //</editor-fold>
    //<editor-fold desc="Krnl">
    let krnl = CleanerData {
        path: "C:\\exechack\\*".parse().unwrap(),
        program: "Krnl".parse().unwrap(),
        files_to_remove: vec![],
        category: "Cheats".parse().unwrap(),
        remove_directories: true,
        remove_files: true,
        directories_to_remove: vec![],
        remove_all_in_dir: false,
        remove_directory_after_clean: true,
        folders_to_remove: vec![]
    };
    database.push(krnl);
    //</editor-fold>

    //</editor-fold>

    for data in database.iter().clone() {
        if !options.contains(&&*data.category) {
            options.push(&*data.category);
        }
        if !programs.contains(&&*data.program) {
            programs.push(&*data.program);
        }

    }
    println!("DataBase Programs: {}", programs.iter().count());
    let validator = |a: &[ListOption<&&str>]| {
        if a.len() < 1 {
            return Ok(Validation::Invalid("No category is selected!".into()));
        }
        else {
            return Ok(Validation::Valid);
        }
    };

    let formatter: MultiOptionFormatter<'_, &str> = &|a| format!("{} selected categories", a.len());
    let ans = MultiSelect::new("Select the clearing categories:", options)
        .with_validator(validator)
        .with_formatter(formatter)
        .prompt();

    let mut bytes_cleared = 0;
    let mut removed_files = 0;
    let mut removed_directories = 0;
    let mut cleared_programs:Vec<Cleared> = vec![];

    let database2 = database.iter().cloned();
    if let Ok(ans) = ans {
        let async_list: Vec<_> = database2
            .filter(|data| ans.contains(&&*data.category)) // Убедитесь, что ans содержит правильные значения
            .map(|data| {
                let data = Arc::new(data.clone()); // Клонируем значение и оборачиваем в Arc
                task::spawn(async move {
                    clear_category(&data) // Предполагаем, что clear_category асинхронный
                })
            })
            .collect();

        for async_task in async_list {
            match async_task.await {
                Ok(result) => {
                    removed_files += result.files;
                    removed_directories += result.folders;
                    bytes_cleared += result.bytes;
                    if result.working {
                        let data2 = Cleared { Program: result.program };
                        if !cleared_programs.contains(&data2) {
                            cleared_programs.push(data2);
                        }
                    }
                },
                Err(_) => {
                    eprintln!("Error waiting for task completion");
                }
            }
        }
    }

    println!("Cleared programms:");
    let table = Table::new(cleared_programs).to_string();
    println!("{}", table);
    println!("Removed: {}", get_file_size_string(bytes_cleared));
    println!("Removed files: {}", removed_files);
    println!("Removed directories: {}", removed_directories);
    let mut s=String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");




}
