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
