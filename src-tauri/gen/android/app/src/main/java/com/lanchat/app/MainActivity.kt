package com.lanchat.app

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.content.FileProvider
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

class MainActivity : TauriActivity() {
    private var pendingSharedFiles: List<SharedFileInfo>? = null
    private var webView: WebView? = null
    private var shareReceiver: BroadcastReceiver? = null

    data class SharedFileInfo(
        val uri: Uri,
        val fileName: String,
        val fileSize: Long,
        val mimeType: String?
    )

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        
        // 注册广播接收器
        registerShareReceiver()
        
        // 延迟注册 JavaScript 接口，等待 WebView 初始化
        window.decorView.post {
            setupJavaScriptInterface()
            // 只在首次启动时检查分享数据（处理冷启动场景）
            checkShareData()
        }
    }
    
    private fun registerShareReceiver() {
        shareReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                println("[MainActivity] 收到分享广播")
                checkShareData()
            }
        }
        
        val filter = IntentFilter("com.lanchat.app.SHARE_RECEIVED")
        registerReceiver(shareReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
        println("[MainActivity] 广播接收器已注册")
    }
    
    override fun onDestroy() {
        super.onDestroy()
        // 注销广播接收器
        shareReceiver?.let {
            unregisterReceiver(it)
            println("[MainActivity] 广播接收器已注销")
        }
    }

    private fun setupJavaScriptInterface() {
        try {
            // 查找 WebView
            webView = findWebView(window.decorView)
            if (webView != null) {
                // 启用 JavaScript
                webView?.settings?.javaScriptEnabled = true
                // 添加 JavaScript 接口
                webView?.addJavascriptInterface(this, "Android")
                println("[MainActivity] JavaScript 接口已注册，WebView: ${webView?.javaClass?.name}")
                
                // 验证接口是否可用
                webView?.evaluateJavascript(
                    "typeof window.Android !== 'undefined'",
                    { result ->
                        println("[MainActivity] window.Android 可用性检查: $result")
                    }
                )
            } else {
                println("[MainActivity] 无法找到 WebView，延迟重试")
                // 延迟重试
                window.decorView.postDelayed({
                    webView = findWebView(window.decorView)
                    if (webView != null) {
                        webView?.settings?.javaScriptEnabled = true
                        webView?.addJavascriptInterface(this, "Android")
                        println("[MainActivity] JavaScript 接口已注册（延迟），WebView: ${webView?.javaClass?.name}")
                        
                        webView?.evaluateJavascript(
                            "typeof window.Android !== 'undefined'",
                            { result ->
                                println("[MainActivity] window.Android 可用性检查（延迟）: $result")
                            }
                        )
                    } else {
                        println("[MainActivity] 仍无法找到 WebView")
                    }
                }, 1000)
            }
        } catch (e: Exception) {
            println("[MainActivity] 注册 JavaScript 接口失败: ${e.message}")
            e.printStackTrace()
        }
    }

    private fun findWebView(view: android.view.View): WebView? {
        println("[MainActivity] 检查 View: ${view.javaClass.name}")
        
        if (view is WebView) {
            println("[MainActivity] 找到 WebView: ${view.javaClass.name}")
            return view
        }
        if (view is android.view.ViewGroup) {
            for (i in 0 until view.childCount) {
                val child = view.getChildAt(i)
                val result = findWebView(child)
                if (result != null) {
                    return result
                }
            }
        }
        return null
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
    }
    
    override fun onResume() {
        super.onResume()
        println("[MainActivity] onResume 被调用")
        // 只在启动时检查分享数据，不在每次 resume 时检查
        // 因为广播接收器会处理运行时的分享
    }
    
    private fun checkShareData() {
        val sharedFiles = ShareDataHolder.sharedFiles
        if (sharedFiles != null && sharedFiles.isNotEmpty()) {
            println("[MainActivity] 检测到分享数据: ${sharedFiles.size} 个文件")
            
            // 转换为 MainActivity 的数据格式
            val fileInfos = sharedFiles.map { file ->
                SharedFileInfo(
                    Uri.parse(file.uri),
                    file.fileName,
                    file.fileSize,
                    file.mimeType
                )
            }
            
            pendingSharedFiles = fileInfos
            println("[MainActivity] 已保存 ${fileInfos.size} 个文件到 pendingSharedFiles")
            
            // 将数据发送到 Rust 层
            sendFilesToRust(sharedFiles)
            
            // 清除全局数据
            ShareDataHolder.sharedFiles = null
            
            // 通知 WebView
            notifyWebView()
        } else {
            println("[MainActivity] 没有分享数据或数据为空")
        }
    }
    
    private fun sendFilesToRust(files: List<ShareActivity.ShareFileInfo>) {
        println("[MainActivity] 准备发送文件到 Rust 层")
        
        // 构造 JSON 数组
        val jsonArray = JSONArray()
        files.forEach { file ->
            val jsonObj = JSONObject().apply {
                put("uri", file.uri)
                put("fileName", file.fileName)
                put("fileSize", file.fileSize)
                put("mimeType", file.mimeType)
                put("fd", file.fd)  // 添加文件描述符
            }
            jsonArray.put(jsonObj)
        }
        
        val jsonString = jsonArray.toString()
        println("[MainActivity] 发送 JSON 到 Rust: $jsonString")
        
        // 通过 JavaScript 调用 Tauri 命令
        runOnUiThread {
            webView?.evaluateJavascript(
                """
                (async function() {
                    try {
                        const files = $jsonString;
                        console.log('[MainActivity->Rust] 设置分享文件:', files);
                        
                        if (window.__TAURI__) {
                            await window.__TAURI__.core.invoke('set_android_shared_files', { files: files });
                            console.log('[MainActivity->Rust] 文件已发送到 Rust');
                        } else {
                            console.error('[MainActivity->Rust] Tauri 不可用');
                        }
                    } catch (e) {
                        console.error('[MainActivity->Rust] 发送失败:', e);
                    }
                })();
                """.trimIndent(),
                null
            )
        }
    }



    private fun notifyWebView() {
        println("[MainActivity] 准备通知 WebView")
        
        // 使用递归重试机制
        notifyWebViewWithRetry(0)
    }
    
    private fun notifyWebViewWithRetry(attempt: Int) {
        val maxAttempts = 10
        val delayMs = 500L
        
        if (attempt >= maxAttempts) {
            println("[MainActivity] 达到最大重试次数，放弃通知")
            return
        }
        
        window.decorView.postDelayed({
            // 尝试重新查找 WebView
            if (webView == null) {
                webView = findWebView(window.decorView)
            }
            
            if (webView != null) {
                println("[MainActivity] WebView 已就绪（尝试 ${attempt + 1}），发送事件")
                runOnUiThread {
                    try {
                        webView?.evaluateJavascript(
                            """
                            (function() {
                                console.log('[MainActivity] 触发 android-share-received 事件');
                                window.dispatchEvent(new CustomEvent('android-share-received'));
                            })();
                            """.trimIndent(),
                            { result ->
                                println("[MainActivity] JavaScript 执行结果: $result")
                            }
                        )
                        println("[MainActivity] 已触发 android-share-received 事件")
                    } catch (e: Exception) {
                        println("[MainActivity] 执行 JavaScript 失败: ${e.message}")
                    }
                }
            } else {
                println("[MainActivity] WebView 未就绪（尝试 ${attempt + 1}），继续重试...")
                notifyWebViewWithRetry(attempt + 1)
            }
        }, delayMs)
    }

    @JavascriptInterface
    fun getPendingSharedFiles(): String {
        val files = pendingSharedFiles
        println("[MainActivity] getPendingSharedFiles 被调用，文件数: ${files?.size ?: 0}")
        
        if (files == null || files.isEmpty()) {
            println("[MainActivity] 返回空数组")
            return "[]"
        }
        
        val jsonArray = JSONArray()
        files.forEach { file ->
            val jsonObj = JSONObject().apply {
                put("uri", file.uri.toString())
                put("fileName", file.fileName)
                put("fileSize", file.fileSize)
                put("mimeType", file.mimeType ?: "application/octet-stream")
            }
            jsonArray.put(jsonObj)
        }
        
        val result = jsonArray.toString()
        println("[MainActivity] 返回 JSON: $result")
        return result
    }

    @JavascriptInterface
    fun clearPendingSharedFiles() {
        pendingSharedFiles = null
    }

    @JavascriptInterface
    fun getFileDescriptor(uriString: String): Int {
        return try {
            val uri = Uri.parse(uriString)
            val pfd = contentResolver.openFileDescriptor(uri, "r")
            if (pfd != null) {
                val fd = pfd.detachFd() // 分离 FD，由 Rust 负责关闭
                println("[MainActivity] 获取文件描述符成功: fd=$fd, uri=$uriString")
                fd
            } else {
                println("[MainActivity] 无法打开文件描述符: $uriString")
                -1
            }
        } catch (e: Exception) {
            println("[MainActivity] 获取文件描述符失败: ${e.message}")
            -1
        }
    }

    // 分享文件到其他应用
    fun shareFile(filePath: String) {
        try {
            println("[MainActivity] 准备分享文件: $filePath")
            
            val uri: Uri
            val mimeType: String
            
            if (filePath.startsWith("content://")) {
                // 已经是 content URI，直接使用
                uri = Uri.parse(filePath)
                mimeType = contentResolver.getType(uri) ?: "*/*"
                println("[MainActivity] 使用 content URI: $uri")
            } else {
                // 普通文件路径，使用 FileProvider
                val file = File(filePath)
                if (!file.exists()) {
                    println("[MainActivity] 文件不存在: $filePath")
                    return
                }
                
                uri = FileProvider.getUriForFile(
                    this,
                    "${applicationContext.packageName}.fileprovider",
                    file
                )
                mimeType = contentResolver.getType(uri) ?: "*/*"
                println("[MainActivity] FileProvider URI: $uri")
            }
            
            println("[MainActivity] MIME 类型: $mimeType")
            
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = mimeType
                putExtra(Intent.EXTRA_STREAM, uri)
                // 添加读写权限标志
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            }
            
            // 创建分享选择器并授予权限
            val chooser = Intent.createChooser(intent, "分享文件").apply {
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            }
            
            // 显示分享选择器
            startActivity(chooser)
            println("[MainActivity] 分享选择器已启动")
        } catch (e: Exception) {
            println("[MainActivity] 分享文件失败: ${e.message}")
            e.printStackTrace()
        }
    }
}
