package com.lanchat.app

import android.app.Activity
import android.app.ActivityManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.view.ViewGroup
import android.widget.*

/**
 * 独立的分享处理 Activity
 * 检测主应用是否运行，如果运行则直接传递数据，否则启动主应用
 */
class ShareActivity : Activity() {
    
    private var sharedFiles: List<ShareFileInfo> = emptyList()
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        println("[ShareActivity] 收到分享 Intent")
        
        // 处理分享数据
        if (!handleShareIntent(intent)) {
            finish()
            return
        }
        
        // 保存分享数据到全局单例
        ShareDataHolder.sharedFiles = sharedFiles
        
        // 检查主应用是否在运行
        if (isMainActivityRunning()) {
            println("[ShareActivity] 主应用正在运行，直接传递数据")
            // 主应用在运行，直接通知它
            notifyMainActivity()
            finish()
        } else {
            println("[ShareActivity] 主应用未运行，启动主应用")
            // 主应用未运行，启动它
            launchMainApp()
            finish()
        }
    }
    
    private fun isMainActivityRunning(): Boolean {
        val activityManager = getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val runningTasks = activityManager.appTasks
        
        for (task in runningTasks) {
            val taskInfo = task.taskInfo
            if (taskInfo.baseActivity?.className == "com.lanchat.app.MainActivity") {
                println("[ShareActivity] 检测到 MainActivity 正在运行")
                return true
            }
        }
        
        println("[ShareActivity] MainActivity 未运行")
        return false
    }
    
    private fun notifyMainActivity() {
        // 发送广播通知 MainActivity
        val intent = Intent("com.lanchat.app.SHARE_RECEIVED")
        sendBroadcast(intent)
        
        // 同时尝试将 MainActivity 带到前台
        val mainIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_REORDER_TO_FRONT
        }
        startActivity(mainIntent)
    }
    
    private fun launchMainApp() {
        println("[ShareActivity] 启动主应用")
        
        // 启动主 Activity
        val mainIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        startActivity(mainIntent)
    }
    
    private fun handleShareIntent(intent: Intent?): Boolean {
        if (intent == null) return false
        
        val action = intent.action
        val type = intent.type
        
        println("[ShareActivity] Action: $action, Type: $type")
        
        when (action) {
            Intent.ACTION_SEND -> {
                if (type != null) {
                    val uri = intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
                    if (uri != null) {
                        val fileInfo = getFileInfo(uri, type)
                        if (fileInfo != null) {
                            sharedFiles = listOf(fileInfo)
                            return true
                        }
                    }
                }
            }
            Intent.ACTION_SEND_MULTIPLE -> {
                if (type != null) {
                    val uris = intent.getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM)
                    if (uris != null && uris.isNotEmpty()) {
                        val fileInfos = uris.mapNotNull { getFileInfo(it, type) }
                        if (fileInfos.isNotEmpty()) {
                            sharedFiles = fileInfos
                            return true
                        }
                    }
                }
            }
        }
        
        return false
    }
    
    private fun getFileInfo(uri: Uri, mimeType: String): ShareFileInfo? {
        return try {
            var fileName = "unknown"
            var fileSize = 0L
            
            // 尝试从 ContentResolver 获取文件信息
            contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                    
                    if (nameIndex != -1) {
                        val name = cursor.getString(nameIndex)
                        if (!name.isNullOrBlank()) {
                            fileName = name
                        }
                    }
                    if (sizeIndex != -1) {
                        fileSize = cursor.getLong(sizeIndex)
                    }
                }
            }
            
            // 如果文件名仍然是 unknown，尝试从 URI 中提取
            if (fileName == "unknown") {
                fileName = uri.lastPathSegment ?: "unknown"
                // 如果还是没有扩展名，根据 MIME 类型添加
                if (!fileName.contains(".")) {
                    val extension = when {
                        mimeType.startsWith("image/") -> when (mimeType) {
                            "image/jpeg" -> ".jpg"
                            "image/png" -> ".png"
                            "image/gif" -> ".gif"
                            "image/webp" -> ".webp"
                            else -> ".img"
                        }
                        mimeType.startsWith("video/") -> ".mp4"
                        mimeType.startsWith("audio/") -> ".mp3"
                        mimeType == "application/pdf" -> ".pdf"
                        mimeType.startsWith("text/") -> ".txt"
                        else -> ".dat"
                    }
                    fileName = "shared_file_${System.currentTimeMillis()}$extension"
                }
            }
            
            // 获取文件描述符
            val fd = try {
                val pfd = contentResolver.openFileDescriptor(uri, "r")
                if (pfd != null) {
                    val detachedFd = pfd.detachFd()
                    println("[ShareActivity] 成功获取文件描述符: fd=$detachedFd")
                    detachedFd
                } else {
                    println("[ShareActivity] 无法打开文件描述符")
                    -1
                }
            } catch (e: Exception) {
                println("[ShareActivity] 获取文件描述符失败: ${e.message}")
                -1
            }
            
            println("[ShareActivity] 文件: $fileName, 大小: $fileSize, URI: $uri, FD: $fd")
            ShareFileInfo(uri.toString(), fileName, fileSize, mimeType, fd)
        } catch (e: Exception) {
            println("[ShareActivity] 获取文件信息失败: ${e.message}")
            null
        }
    }
    
    data class ShareFileInfo(
        val uri: String,
        val fileName: String,
        val fileSize: Long,
        val mimeType: String,
        val fd: Int  // 文件描述符
    )
}

/**
 * 全局单例，用于在 Activity 之间传递分享数据
 */
object ShareDataHolder {
    var sharedFiles: List<ShareActivity.ShareFileInfo>? = null
}


