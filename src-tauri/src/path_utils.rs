use dirs::*;
/// 统一的跨平台路径处理工具
///
/// 提供跨平台兼容的路径处理方法，避免硬编码路径
use std::path::{Path, PathBuf};

/// 应用程序相关路径管理器
pub struct AppPaths;

impl AppPaths {
    /// 获取 Antigravity 数据目录
    ///
    /// 跨平台路径:
    /// - Windows: %APPDATA%\Antigravity\User\globalStorage\
    /// - macOS: ~/Library/Application Support/Antigravity/User/globalStorage/
    /// - Linux: ~/.config/Antigravity/User/globalStorage/
    pub fn antigravity_data_dir() -> Option<PathBuf> {
        let result = match std::env::consts::OS {
            "windows" => Self::windows_antigravity_data_dir(),
            "macos" => Self::macos_antigravity_data_dir(),
            "linux" => Self::linux_antigravity_data_dir(),
            _ => Self::fallback_antigravity_data_dir(),
        };

      match &result {
        Some(path) => {
          let sanitized_path = sanitize_user_path(path);
          tracing::info!("🔍 检测 Antigravity 数据目录: {}", sanitized_path);
        }
        None => tracing::info!("🔍 检测 Antigravity 数据目录: 未找到"),
      }

        result
    }

    /// Windows: %APPDATA%\Antigravity\User\globalStorage\
    fn windows_antigravity_data_dir() -> Option<PathBuf> {
        config_dir()
            .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
    }

    /// macOS: ~/Library/Application Support/Antigravity/User/globalStorage/
    fn macos_antigravity_data_dir() -> Option<PathBuf> {
        data_dir()
            .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
    }

    /// Linux: ~/.config/Antigravity/User/globalStorage/ (优先) 或 ~/.local/share/Antigravity/User/globalStorage/ (备用)
    fn linux_antigravity_data_dir() -> Option<PathBuf> {
        // 优先使用 ~/.config
        config_dir()
            .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
            .or_else(|| {
                // 备用：~/.local/share
                data_dir()
                    .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
            })
    }

    /// 其他系统的备用方案
    fn fallback_antigravity_data_dir() -> Option<PathBuf> {
        data_dir()
            .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
    }

    /// 获取 Antigravity 可执行文件路径
    ///
    /// 跨平台搜索路径:
    /// - Windows: %LOCALAPPDATA%\Programs\Antigravity\Antigravity.exe
    /// - macOS: /Applications/Antigravity.app
    /// - Linux: /usr/bin/antigravity, ~/.local/bin/antigravity
    pub fn antigravity_executable_paths() -> Vec<PathBuf> {
        match std::env::consts::OS {
            "windows" => Self::windows_antigravity_executable_paths(),
            "macos" => Self::macos_antigravity_app_paths(),
            "linux" => Self::linux_antigravity_executable_paths(),
            _ => Vec::new(),
        }
    }

    /// Windows Antigravity 可执行文件路径
    fn windows_antigravity_executable_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 用户程序目录: %LOCALAPPDATA%\Programs\
        if let Some(local_data) = data_local_dir() {
            paths.push(local_data.join("Programs").join("Antigravity").join("Antigravity.exe"));
        }

        // 用户数据目录的其他位置
        if let Some(home) = home_dir() {
            // %APPDATA%\Local\Programs\Antigravity\
            paths.push(home.join("AppData").join("Local").join("Programs").join("Antigravity").join("Antigravity.exe"));

            // %APPDATA%\Roaming\Local\Programs\Antigravity\ (虽然不常见，但有些应用会这样安装)
            paths.push(home.join("AppData").join("Roaming").join("Local").join("Programs").join("Antigravity").join("Antigravity.exe"));
        }

        // 系统程序目录
        if let Some(program_files) = Self::get_program_files_dir() {
            paths.push(program_files.join("Antigravity").join("Antigravity.exe"));
        }

        if let Some(program_files_x86) = Self::get_program_files_x86_dir() {
            paths.push(program_files_x86.join("Antigravity").join("Antigravity.exe"));
        }

        paths
    }

    /// macOS Antigravity .app 包路径
    fn macos_antigravity_app_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let app_names = [
            "Antigravity.app",
            "Antigravity-electron.app",
            "Antigravity-alpha.app",
            "Antigravity-beta.app"
        ];

        // 系统应用程序目录
        if let Some(applications) = Self::get_applications_dir() {
            for app_name in &app_names {
                paths.push(applications.join(app_name));
            }
        }

        // 用户应用程序目录
        if let Some(home) = home_dir() {
            let user_apps = home.join("Applications");
            for app_name in &app_names {
                paths.push(user_apps.join(app_name));
            }
        }

        paths
    }

    /// Linux Antigravity 可执行文件路径
    fn linux_antigravity_executable_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 系统二进制目录
        paths.push(PathBuf::from("/usr/bin/antigravity"));
        paths.push(PathBuf::from("/usr/local/bin/antigravity"));
        paths.push(PathBuf::from("/usr/share/antigravity/antigravity"));

        // 用户二进制目录
        if let Some(home) = home_dir() {
            paths.push(home.join(".local").join("bin").join("antigravity"));
            paths.push(home.join("bin").join("antigravity"));
        }

        // Snap 包
        paths.push(PathBuf::from("/snap/bin/antigravity"));

        // AppImage 和 Flatpak
        if let Some(home) = home_dir() {
            paths.push(home.join("Applications").join("Antigravity.AppImage"));
        }

        // Flatpak
        paths.push(PathBuf::from("/var/lib/flatpak/exports/bin/antigravity"));
        if let Some(home) = home_dir() {
            paths.push(home.join(".local").join("share").join("flatpak").join("exports").join("bin").join("antigravity"));
        }

        paths
    }

    /// 获取配置目录
    ///
    /// 统一的配置目录获取，避免硬编码
    pub fn config_dir() -> Option<PathBuf> {
        config_dir()
            .map(|path| path.join(".antigravity-agent"))
    }

    /// 获取备份目录
    pub fn backup_dir() -> Option<PathBuf> {
        Self::config_dir().map(|path| path.join("antigravity-accounts"))
    }

    // Windows 特定的辅助方法
    #[cfg(target_os = "windows")]
    fn get_program_files_dir() -> Option<PathBuf> {
        std::env::var("ProgramFiles")
            .ok()
            .map(PathBuf::from)
    }

    #[cfg(target_os = "windows")]
    fn get_program_files_x86_dir() -> Option<PathBuf> {
        std::env::var("ProgramFiles(x86)")
            .ok()
            .map(PathBuf::from)
    }

    // macOS 特定的辅助方法
    #[cfg(target_os = "macos")]
    fn get_applications_dir() -> Option<PathBuf> {
        Some(PathBuf::from("/Applications"))
    }

    // Linux 特定的辅助方法（空实现，因为 Linux 使用硬编码路径）
    #[cfg(not(target_os = "windows"))]
    fn get_program_files_dir() -> Option<PathBuf> {
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn get_program_files_x86_dir() -> Option<PathBuf> {
        None
    }

    #[cfg(not(target_os = "macos"))]
    fn get_applications_dir() -> Option<PathBuf> {
        None
    }
}

/// 跨平台路径脱敏函数
/// 将用户名替换为 ****，支持 Windows、macOS、Linux
fn sanitize_user_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    if std::env::consts::OS == "windows" {
        // Windows: C:\Users\Kiki\AppData\Roaming\... -> C:\Users\****\AppData\Roaming\...
        if let Some(start) = path_str.find("\\Users\\") {
            let user_path_start = start + 7; // 跳过 "\Users\"
            if let Some(end) = path_str[user_path_start..].find('\\') {
                let end = user_path_start + end;
                return format!("{}\\Users\\****\\{}", &path_str[..start], &path_str[end + 1..]);
            }
        }
    } else if std::env::consts::OS == "macos" {
        // macOS: /Users/kiki/Library/Application Support/... -> /Users/****/Library/Application Support/...
        if let Some(start) = path_str.find("/Users/") {
            let user_path_start = start + 7; // 跳过 "/Users/"
            if let Some(end) = path_str[user_path_start..].find('/') {
                let end = user_path_start + end;
                return format!("{}/Users/****/{}", &path_str[..start], &path_str[end + 1..]);
            }
        }
    } else if std::env::consts::OS == "linux" {
        // Linux: /home/user/.config/... -> /home/****/.config/...
        if let Some(start) = path_str.find("/home/") {
            let user_path_start = start + 6; // 跳过 "/home/"
            if let Some(end) = path_str[user_path_start..].find('/') {
                let end = user_path_start + end;
                return format!("{}/home/****/{}", &path_str[..start], &path_str[end + 1..]);
            }
        }
    }

    // 如果没有匹配到任何模式，返回原路径
    path_str.to_string()
}

