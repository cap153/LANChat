// android_fd.rs - Android 文件描述符处理
#[cfg(target_os = "android")]
use std::os::unix::io::{FromRawFd, RawFd};

#[cfg(target_os = "android")]
pub struct AndroidFile {
    file: std::fs::File,
}

#[cfg(target_os = "android")]
impl AndroidFile {
    /// 从文件描述符创建 File 对象
    pub fn from_fd(fd: RawFd) -> Result<Self, String> {
        if fd < 0 {
            return Err("无效的文件描述符".to_string());
        }

        println!("[AndroidFD] 从 FD 创建文件对象: fd={}", fd);

        // SAFETY: 我们假设 FD 是有效的，由 Android ContentResolver 提供
        let file = unsafe { std::fs::File::from_raw_fd(fd) };

        Ok(AndroidFile { file })
    }

    /// 获取内部的 File 对象
    pub fn into_file(self) -> std::fs::File {
        self.file
    }
    
    /// 从 content:// URI 获取文件描述符
    /// 这需要通过 JNI 调用 Android 的 ContentResolver
    pub fn from_content_uri(uri: &str) -> Result<Self, String> {
        println!("[AndroidFD] 尝试从 content URI 获取 FD: {}", uri);
        
        // 使用 ndk-context 获取 Android 上下文
        use jni::objects::{JObject, JValue};
        use jni::JavaVM;
        
        // 获取 JNI 环境
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("无法获取 JavaVM: {}", e))?;
        
        let mut env = vm.attach_current_thread()
            .map_err(|e| format!("无法附加到当前线程: {}", e))?;
        
        // 获取 Context 对象
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };
        
        // 调用 ContentResolver
        let content_resolver = env.call_method(
            context,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[]
        ).map_err(|e| format!("无法获取 ContentResolver: {}", e))?
        .l().map_err(|e| format!("无法转换为对象: {}", e))?;
        
        // 创建 URI 对象
        let uri_string = env.new_string(uri)
            .map_err(|e| format!("无法创建 URI 字符串: {}", e))?;
        
        let uri_obj = env.call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&JObject::from(uri_string))]
        ).map_err(|e| format!("无法解析 URI: {}", e))?
        .l().map_err(|e| format!("无法转换为对象: {}", e))?;
        
        // 打开文件描述符
        let mode_string = env.new_string("r")
            .map_err(|e| format!("无法创建模式字符串: {}", e))?;
        
        let pfd = env.call_method(
            content_resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[
                JValue::Object(&uri_obj),
                JValue::Object(&JObject::from(mode_string))
            ]
        ).map_err(|e| format!("无法打开文件描述符: {}", e))?
        .l().map_err(|e| format!("无法转换为对象: {}", e))?;
        
        // 获取文件描述符
        let fd = env.call_method(
            pfd,
            "detachFd",
            "()I",
            &[]
        ).map_err(|e| format!("无法分离文件描述符: {}", e))?
        .i().map_err(|e| format!("无法转换为整数: {}", e))?;
        
        println!("[AndroidFD] 成功获取文件描述符: fd={}", fd);
        
        Self::from_fd(fd)
    }
}

#[cfg(not(target_os = "android"))]
pub struct AndroidFile;

#[cfg(not(target_os = "android"))]
impl AndroidFile {
    pub fn from_fd(_fd: i32) -> Result<Self, String> {
        Err("此功能仅在 Android 上可用".to_string())
    }

    pub fn into_file(self) -> std::fs::File {
        panic!("此功能仅在 Android 上可用")
    }
    
    pub fn from_content_uri(_uri: &str) -> Result<Self, String> {
        Err("此功能仅在 Android 上可用".to_string())
    }
}
