#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|_app| {
            tauri::async_runtime::spawn(async {
                let addr: SocketAddr = "127.0.0.1:8080".parse().expect("valid addr");
                if let Err(err) = server::run_http(addr).await {
                    tracing::error!("server failed: {err}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
