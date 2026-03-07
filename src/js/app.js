// src/js/app.js
async function renderPage() {
    console.log("[JS-App] 页面初始化开始...");
    
    const myName = await apiGetMyName();
    const nameElement = document.getElementById('my-name');
    if (nameElement) {
        nameElement.innerText = myName;
    }

    // 初始化改名功能
    initNameEditor();
    
    // 初始化设置功能
    initSettings();
    
    // 初始化主题功能
    initTheme();
    
    // 初始化聊天功能
    initChat();

    // 使用我们封装好的 apiListen
    await apiListen('new-peer', (event) => {
        console.log("[JS-App] 收到新邻居:", event.payload);
        addUserToList(event.payload.id, event.payload.name, event.payload.addr, false);
    });

    // 监听新消息事件(桌面端)
    await apiListen('new-message', (event) => {
        console.log("[JS-App] ========== 收到 new-message 事件 ==========");
        console.log("[JS-App] 事件类型:", typeof event);
        console.log("[JS-App] 事件对象:", event);
        console.log("[JS-App] payload 类型:", typeof event.payload);
        console.log("[JS-App] payload 内容:", JSON.stringify(event.payload, null, 2));
        console.log("[JS-App] ==========================================");
        onReceiveMessage(event.payload);
    });

    // 启动用户列表轮询（桌面端和 Web 端都需要）
    console.log("[JS-App] 启动用户列表轮询");
    startPeerPolling();

    // 启动消息轮询（桌面端和 Web 端都需要，用于检测状态变化）
    const tauri = window.__TAURI__;
    console.log("[JS-App] 启动消息轮询");
    startMessagePolling();
}

// Web 端轮询用户列表
async function startPeerPolling() {
    const pollInterval = 3000; // 3秒轮询一次
    
    const updatePeerList = async () => {
        const peers = await apiGetPeers();
        
        // 获取当前列表中的所有 ID
        const currentIds = new Set();
        const list = document.getElementById('user-list');
        if (list) {
            const items = list.querySelectorAll('li');
            items.forEach(item => currentIds.add(item.dataset.id));
        }
        
        // 更新用户列表
        const receivedIds = new Set();
        for (const peer of peers) {
            addUserToList(peer.id, peer.name, peer.addr, peer.is_offline);
            receivedIds.add(peer.id);
        }
        
        // 移除不在服务器列表中的用户（已经超过60秒）
        for (const id of currentIds) {
            if (!receivedIds.has(id)) {
                removeUserFromList(id);
            }
        }
    };
    
    // 立即执行一次
    await updatePeerList();
    
    // 定时轮询
    setInterval(updatePeerList, pollInterval);
}

document.addEventListener('DOMContentLoaded', renderPage);

// 监听 Android 分享事件
window.addEventListener('android-share-received', async () => {
    console.log("[JS-App] ========== 收到 Android 分享事件 ==========");
    
    try {
        const sharedFiles = await apiGetAndroidSharedFiles();
        console.log("[JS-App] 分享的文件:", sharedFiles);
        
        if (sharedFiles.length === 0) {
            console.log("[JS-App] 没有待处理的分享文件");
            return;
        }
        
        console.log("[JS-App] 准备显示分享对话框");
        // 显示在线用户选择弹窗
        showShareDialog(sharedFiles);
    } catch (e) {
        console.error("[JS-App] 处理 Android 分享失败:", e);
    }
});

console.log("[JS-App] Android 分享事件监听器已注册");

// 显示分享对话框
function showShareDialog(sharedFiles) {
    console.log("[JS-App] showShareDialog 被调用，文件数:", sharedFiles.length);
    
    // 创建弹窗
    const dialog = document.createElement('div');
    dialog.className = 'share-dialog';
    dialog.innerHTML = `
        <div class="share-dialog-content">
            <h3>选择接收者</h3>
            <p>共 ${sharedFiles.length} 个文件</p>
            <ul class="share-user-list" id="share-user-list"></ul>
            <button class="cancel-btn" onclick="closeShareDialog()">取消</button>
        </div>
    `;
    
    document.body.appendChild(dialog);
    console.log("[JS-App] 对话框已添加到 DOM");
    
    // 填充在线用户列表（只显示非 offline 的用户）
    const userList = document.getElementById('share-user-list');
    const allUsers = document.querySelectorAll('#user-list li');
    
    console.log("[JS-App] 找到用户列表项:", allUsers.length);
    
    let onlineCount = 0;
    allUsers.forEach(userItem => {
        const isOffline = userItem.classList.contains('offline');
        console.log("[JS-App] 用户:", userItem.dataset.name, "offline:", isOffline);
        
        if (!isOffline) {
            onlineCount++;
            const userId = userItem.dataset.id;
            const userName = userItem.dataset.name;
            const userAddr = userItem.dataset.addr;
            
            const li = document.createElement('li');
            li.textContent = userName;
            li.onclick = () => handleShareToUser(userId, userName, userAddr, sharedFiles);
            userList.appendChild(li);
        }
    });
    
    console.log("[JS-App] 在线用户数:", onlineCount);
    
    if (userList.children.length === 0) {
        userList.innerHTML = '<li class="no-users">暂无在线用户</li>';
        console.log("[JS-App] 没有在线用户");
    }
}

// 关闭分享对话框
function closeShareDialog() {
    const dialog = document.querySelector('.share-dialog');
    if (dialog) {
        dialog.remove();
    }
    apiClearAndroidSharedFiles();
}

// 处理分享到指定用户
async function handleShareToUser(userId, userName, userAddr, sharedFiles) {
    console.log("[JS-App] 分享文件到:", userName);
    
    // 关闭对话框
    closeShareDialog();
    
    // 打开该用户的聊天界面
    openChat({ id: userId, name: userName, addr: userAddr });
    
    // 发送所有文件
    for (const fileInfo of sharedFiles) {
        try {
            console.log("[JS-App] 发送文件:", fileInfo.fileName);
            
            // 在聊天界面显示上传中状态
            const chatBox = document.getElementById('chat-box');
            if (chatBox) {
                const msgDiv = document.createElement('div');
                msgDiv.className = 'message sent';
                msgDiv.innerHTML = `
                    <div class="file-message">
                        <span class="file-name">${fileInfo.fileName}</span>
                        <span class="file-size">${formatFileSize(fileInfo.fileSize)}</span>
                        <span class="file-status file-uploading">上传中...</span>
                    </div>
                `;
                chatBox.appendChild(msgDiv);
                chatBox.scrollTop = chatBox.scrollHeight;
            }
            
            // 发送文件
            await apiSendFileFromAndroidUri(userId, userAddr, fileInfo);
            
            console.log("[JS-App] 文件发送成功:", fileInfo.fileName);
            
            // 更新状态为已发送
            const statusSpans = document.querySelectorAll('.file-uploading');
            statusSpans.forEach(span => {
                if (span.previousElementSibling?.textContent === formatFileSize(fileInfo.fileSize)) {
                    span.textContent = '已发送';
                    span.classList.remove('file-uploading');
                }
            });
        } catch (e) {
            console.error("[JS-App] 文件发送失败:", fileInfo.fileName, e);
            alert(`发送文件失败: ${fileInfo.fileName}\n${e.message}`);
        }
    }
    
    // 清除待处理的分享文件
    apiClearAndroidSharedFiles();
}

// 格式化文件大小
function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
}


document.addEventListener('DOMContentLoaded', renderPage);


// Web 端轮询新消息
async function startMessagePolling() {
    const pollInterval = 2000; // 2秒轮询一次
    
    const checkNewMessages = async () => {
        if (!window.currentChatPeer) return;
        
        try {
            const messages = await apiGetChatHistory(window.currentChatPeer.id);
            
            // 检查是否有新消息或状态变化
            let hasChanges = false;
            
            // 检查消息数量是否变化
            const chatMessages = document.getElementById('chat-messages');
            const currentMessageCount = chatMessages.querySelectorAll('.message').length;
            
            if (messages.length !== currentMessageCount) {
                hasChanges = true;
            } else {
                // 检查是否有文件状态变化（downloading -> pending/accepted）
                for (const msg of messages) {
                    if (msg.msg_type === 'file' && msg.file_status !== 'downloading') {
                        // 检查当前显示的消息是否还是 downloading 状态
                        const fileMessages = chatMessages.querySelectorAll('.file-downloading');
                        if (fileMessages.length > 0) {
                            hasChanges = true;
                            break;
                        }
                    }
                }
            }
            
            // 如果有变化，刷新整个聊天历史
            if (hasChanges) {
                console.log('[JS-App] 检测到消息变化，刷新聊天历史');
                await loadChatHistory(window.currentChatPeer.id, true); // 保持滚动位置
            }
        } catch (e) {
            console.error('[JS-App] 轮询消息失败:', e);
        }
    };
    
    // 定时轮询
    setInterval(checkNewMessages, pollInterval);
}
