//! 协议转换

use crate::error::Result;

/// 协议转换器
pub struct ProtocolConverter;

impl ProtocolConverter {
    /// 转换为设备原生协议
    pub fn to_native(command: &str, device_type: &str) -> Result<String> {
        // TODO: 实现协议转换
        Ok(command.to_string())
    }

    /// 从设备原生协议转换
    pub fn from_native(response: &str) -> Result<String> {
        Ok(response.to_string())
    }
}
