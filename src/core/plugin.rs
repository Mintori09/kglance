use crate::core::config::{CliPluginConfig, NativePluginConfig};
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::c_char;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub enum PluginOutputType {
    Text,
    Image,
    Html,
}

#[derive(Debug, Clone)]
pub struct PluginOutput {
    pub output_type: PluginOutputType,
    pub data: Vec<u8>,
}

pub struct PluginManager;

impl PluginManager {
    pub async fn run_cli_plugin(
        config: &CliPluginConfig,
        file_path: &str,
        timeout_ms: u64,
    ) -> Result<PluginOutput, String> {
        let args: Vec<String> = config
            .args
            .iter()
            .map(|arg| arg.replace("{file}", file_path))
            .collect();

        let mut cmd = Command::new(&config.command);
        cmd.args(&args);

        let output = tokio::time::timeout(Duration::from_millis(timeout_ms), cmd.output())
            .await
            .map_err(|_| "Plugin execution timed out".to_string())?
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Command failed with code {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_type = match config.output_type.to_lowercase().as_str() {
            "image" => PluginOutputType::Image,
            "html" => PluginOutputType::Html,
            _ => PluginOutputType::Text,
        };

        Ok(PluginOutput {
            output_type,
            data: output.stdout,
        })
    }

    /// Executing native C ABI shared library plugins
    /// # Safety
    /// Caller must ensure `config.library_path` points to a valid C dynamic library.
    pub unsafe fn run_native_plugin(
        config: &NativePluginConfig,
        file_path: &str,
    ) -> Result<PluginOutput, String> {
        unsafe {
            let lib = Library::new(&config.library_path).map_err(|e| e.to_string())?;
            type PluginFunc = unsafe extern "C" fn(file_path: *const c_char) -> *mut c_char;

            let func: Symbol<PluginFunc> = lib
                .get(b"kglance_plugin_preview\0")
                .map_err(|e| e.to_string())?;

            let c_path = CString::new(file_path).map_err(|e| e.to_string())?;
            let result_ptr = func(c_path.as_ptr());

            if result_ptr.is_null() {
                return Err("Native plugin returned null result".to_string());
            }

            let c_str = std::ffi::CStr::from_ptr(result_ptr);
            let bytes = c_str.to_bytes().to_vec();

            Ok(PluginOutput {
                output_type: PluginOutputType::Text,
                data: bytes,
            })
        }
    }
}
