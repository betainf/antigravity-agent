// 窗口事件处理模块
// 负责在应用启动时恢复窗口状态

use tauri::Manager;
use crate::window_state_manager::{WindowState, load_window_state, save_window_state};

/// 初始化窗口事件处理器
pub fn init_window_event_handler(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 获取主窗口
    let main_window = app.get_webview_window("main")
        .ok_or("无法获取主窗口")?;

    // 应用启动时，尝试恢复上次保存的窗口状态
    let window_clone = main_window.clone();
    tokio::spawn(async move {
        if let Ok(saved_state) = load_window_state().await {
            println!("🔄 恢复窗口状态: 位置({:.1}, {:.1}), 大小({:.1}x{:.1}), 最大化:{}",
                     saved_state.x, saved_state.y, saved_state.width, saved_state.height, saved_state.maximized);

            // 设置窗口位置和大小
            let _ = window_clone.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition {
                    x: saved_state.x as i32,
                    y: saved_state.y as i32,
                }
            ));

            let _ = window_clone.set_size(tauri::Size::Physical(
                tauri::PhysicalSize {
                    width: saved_state.width as u32,
                    height: saved_state.height as u32,
                }
            ));

            // 如果之前是最大化状态，则恢复最大化
            if saved_state.maximized {
                let _ = window_clone.maximize();
            }

            println!("✅ 窗口状态恢复完成");
        }
    });

    // 监听窗口关闭事件，保存当前状态
    let window_clone = main_window.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            let window = window_clone.clone();
            tokio::spawn(async move {
                if let (Ok(outer_position), Ok(outer_size), Ok(is_maximized)) = (
                    window.outer_position(),
                    window.outer_size(),
                    window.is_maximized()
                ) {
                    let current_state = WindowState {
                        x: outer_position.x as f64,
                        y: outer_position.y as f64,
                        width: outer_size.width as f64,
                        height: outer_size.height as f64,
                        maximized: is_maximized,
                    };

                    if let Err(e) = save_window_state(current_state).await {
                        eprintln!("窗口关闭时保存状态失败: {}", e);
                    }
                }
            });
        }
    });

    Ok(())
}