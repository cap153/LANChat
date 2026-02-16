// UI 交互逻辑

function initNameEditor() {
    const editBtn = document.getElementById('edit-name-btn');
    const editPanel = document.getElementById('edit-name-panel');
    const nameInput = document.getElementById('new-name-input');
    const saveBtn = document.getElementById('save-name-btn');
    const cancelBtn = document.getElementById('cancel-name-btn');
    const errorMsg = document.getElementById('error-msg');
    const nameDisplay = document.getElementById('my-name');

    // 点击编辑按钮
    editBtn.addEventListener('click', () => {
        editPanel.style.display = 'block';
        nameInput.value = '';
        nameInput.focus();
        errorMsg.textContent = '';
    });

    // 点击取消按钮
    cancelBtn.addEventListener('click', () => {
        editPanel.style.display = 'none';
        errorMsg.textContent = '';
    });

    // 点击保存按钮
    saveBtn.addEventListener('click', async () => {
        const newName = nameInput.value.trim();
        
        if (!newName) {
            errorMsg.textContent = '用户名不能为空';
            return;
        }
        
        if (newName.length > 50) {
            errorMsg.textContent = '用户名过长（最多50个字符）';
            return;
        }

        try {
            saveBtn.disabled = true;
            saveBtn.textContent = '保存中...';
            errorMsg.textContent = '';
            
            const updatedName = await apiUpdateMyName(newName);
            
            // 更新显示
            nameDisplay.textContent = '我是：' + updatedName;
            editPanel.style.display = 'none';
            
            console.log('[UI] 用户名更新成功:', updatedName);
        } catch (e) {
            errorMsg.textContent = e.message || '更新失败';
            console.error('[UI] 更新用户名失败:', e);
        } finally {
            saveBtn.disabled = false;
            saveBtn.textContent = '保存';
        }
    });

    // 支持回车键保存
    nameInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            saveBtn.click();
        }
    });

    // 支持 ESC 键取消
    nameInput.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            cancelBtn.click();
        }
    });
}

// 添加新用户到列表
function addUserToList(id, name, addr, isOffline = false) {
    const list = document.getElementById('user-list');
    if (!list) return;
    
    // 检查是否已存在
    const existingItems = list.querySelectorAll('li');
    for (let item of existingItems) {
        if (item.dataset.id === id) {
            // 已存在,更新状态
            updateUserStatus(item, name, addr, isOffline);
            return;
        }
    }
    
    // 不存在,创建新的
    const li = document.createElement('li');
    li.dataset.id = id;
    li.innerHTML = `
        <span class="user-name">${name}</span>
        <span class="user-addr">${addr}</span>
        <span class="user-status">${isOffline ? 'offline' : ''}</span>
    `;
    
    if (isOffline) {
        li.classList.add('offline');
    }
    
    // 添加点击事件
    li.addEventListener('click', () => {
        if (!isOffline) {
            openChat({ id, name, addr });
        }
    });
    
    list.appendChild(li);
    
    console.log('[UI] 添加用户到列表:', name, id, isOffline ? '(离线)' : '(在线)');
}

// 更新用户状态
function updateUserStatus(item, name, addr, isOffline) {
    const statusSpan = item.querySelector('.user-status');
    const nameSpan = item.querySelector('.user-name');
    const addrSpan = item.querySelector('.user-addr');
    
    // 更新名字（可能改名了）
    if (nameSpan) {
        nameSpan.textContent = name;
    }
    
    // 更新地址（可能 IP 变了）
    if (addrSpan) {
        addrSpan.textContent = addr;
    }
    
    // 更新离线状态
    if (statusSpan) {
        statusSpan.textContent = isOffline ? 'offline' : '';
    }
    
    if (isOffline) {
        if (!item.classList.contains('offline')) {
            console.log('[UI] 用户离线:', name);
        }
        item.classList.add('offline');
    } else {
        if (item.classList.contains('offline')) {
            console.log('[UI] 用户重新上线:', name);
        }
        item.classList.remove('offline');
    }
}

// 从列表中移除用户
function removeUserFromList(id) {
    const list = document.getElementById('user-list');
    if (!list) return;
    
    const items = list.querySelectorAll('li');
    for (let item of items) {
        if (item.dataset.id === id) {
            const name = item.querySelector('.user-name').textContent;
            item.remove();
            console.log('[UI] 移除用户:', name, id);
            return;
        }
    }
}


// 当前聊天对象 - 全局变量
window.currentChatPeer = null;

// 初始化聊天功能
function initChat() {
    const closeChatBtn = document.getElementById('close-chat-btn');
    const sendBtn = document.getElementById('send-btn');
    const chatInput = document.getElementById('chat-input');
    const attachFileBtn = document.getElementById('attach-file-btn');
    const fileInput = document.getElementById('file-input');
    
    // 关闭聊天窗口
    closeChatBtn.addEventListener('click', () => {
        closeChat();
    });
    
    // 发送消息
    sendBtn.addEventListener('click', () => {
        sendMessage();
    });
    
    // 回车发送
    chatInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    });
    
    // 选择文件
    attachFileBtn.addEventListener('click', () => {
        const tauri = window.__TAURI__;
        if (tauri) {
            // 桌面端 - 直接调用 sendFile，它会弹出对话框
            sendFile(null);
        } else {
            // Web 端 - 触发文件选择
            fileInput.click();
        }
    });
    
    // 文件选择后发送（仅 Web 端）
    fileInput.addEventListener('change', async (e) => {
        const file = e.target.files[0];
        if (file) {
            await sendFile(file);
            fileInput.value = ''; // 清空选择
        }
    });
}

// 打开聊天窗口
function openChat(peer) {
    window.currentChatPeer = peer;
    
    const chatContainer = document.getElementById('chat-container');
    const chatWithName = document.getElementById('chat-with-name');
    const chatMessages = document.getElementById('chat-messages');
    
    chatContainer.style.display = 'flex';
    chatWithName.textContent = `与 ${peer.name} 聊天`;
    chatMessages.innerHTML = '';
    
    // 高亮选中的用户
    const userList = document.getElementById('user-list');
    const items = userList.querySelectorAll('li');
    items.forEach(item => {
        if (item.dataset.id === peer.id) {
            item.classList.add('active');
        } else {
            item.classList.remove('active');
        }
    });
    
    // 重置最后消息时间戳
    window.lastMessageTimestamp = 0;
    
    // 加载历史消息
    loadChatHistory(peer.id);
    
    console.log('[UI] 打开与', peer.name, '的聊天窗口');
}

// 关闭聊天窗口
function closeChat() {
    window.currentChatPeer = null;
    
    const chatContainer = document.getElementById('chat-container');
    chatContainer.style.display = 'none';
    
    // 取消高亮
    const userList = document.getElementById('user-list');
    const items = userList.querySelectorAll('li');
    items.forEach(item => item.classList.remove('active'));
    
    console.log('[UI] 关闭聊天窗口');
}

// 发送消息
async function sendMessage() {
    if (!window.currentChatPeer) return;
    
    const chatInput = document.getElementById('chat-input');
    const content = chatInput.value.trim();
    
    if (!content) return;
    
    try {
        // 调用 API 发送消息
        await apiSendMessage(window.currentChatPeer.id, window.currentChatPeer.addr, content);
        
        // 清空输入框
        chatInput.value = '';
        
        // 显示消息
        addMessageToChat({
            from_id: 'me',
            content: content,
            timestamp: Date.now() / 1000
        }, true);
        
        console.log('[UI] 发送消息:', content);
    } catch (e) {
        console.error('[UI] 发送消息失败:', e);
        alert('发送失败: ' + e.message);
    }
}

// 添加消息到聊天窗口
function addMessageToChat(message, isSent) {
    console.log('[UI] addMessageToChat 被调用');
    console.log('[UI] 消息类型:', message.msg_type);
    console.log('[UI] 是否发送:', isSent);
    
    const chatMessages = document.getElementById('chat-messages');
    
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${isSent ? 'sent' : 'received'}`;
    
    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';
    
    // 检查是否是文件消息
    if (message.msg_type === 'file') {
        console.log('[UI] 渲染文件消息:', message.file_name || message.content);
        const fileDiv = document.createElement('div');
        fileDiv.className = 'message-file';
        fileDiv.innerHTML = `
            <div class="file-info">
                <span class="file-icon">📄</span>
                <div>
                    <div class="file-name">${message.file_name || message.content}</div>
                    <div class="file-size">${message.file_size ? formatFileSize(message.file_size) : '未知大小'}</div>
                </div>
            </div>
        `;
        
        // 如果是接收的文件,添加下载功能
        if (!isSent && message.file_id) {
            fileDiv.style.cursor = 'pointer';
            fileDiv.addEventListener('click', () => {
                downloadFile(message.file_id, message.file_name || message.content);
            });
        }
        
        contentDiv.appendChild(fileDiv);
    } else {
        // 文本消息
        console.log('[UI] 渲染文本消息:', message.content);
        contentDiv.textContent = message.content;
    }
    
    const timeDiv = document.createElement('div');
    timeDiv.className = 'message-time';
    const date = new Date(message.timestamp * 1000);
    timeDiv.textContent = date.toLocaleTimeString();
    
    messageDiv.appendChild(contentDiv);
    messageDiv.appendChild(timeDiv);
    chatMessages.appendChild(messageDiv);
    
    // 滚动到底部
    chatMessages.scrollTop = chatMessages.scrollHeight;
    
    console.log('[UI] 消息已添加到聊天窗口');
}

// 加载聊天历史
async function loadChatHistory(peerId) {
    try {
        const messages = await apiGetChatHistory(peerId);
        
        const chatMessages = document.getElementById('chat-messages');
        chatMessages.innerHTML = '';
        
        for (const msg of messages) {
            addMessageToChat(msg, msg.from_id === 'me');
            // 更新最后消息时间戳
            if (msg.timestamp > (window.lastMessageTimestamp || 0)) {
                window.lastMessageTimestamp = msg.timestamp;
            }
        }
        
        console.log('[UI] 加载了', messages.length, '条历史消息');
    } catch (e) {
        console.error('[UI] 加载历史消息失败:', e);
    }
}

// 接收到新消息
function onReceiveMessage(message) {
    console.log('[UI] 收到新消息:', message);
    console.log('[UI] 当前聊天对象:', window.currentChatPeer);
    
    // 如果正在和发送者聊天,显示消息
    if (window.currentChatPeer && window.currentChatPeer.id === message.from_id) {
        console.log('[UI] 匹配当前聊天对象，显示消息');
        addMessageToChat(message, false);
    } else {
        console.log('[UI] 不匹配当前聊天对象，消息未显示');
    }
    
    // TODO: 显示未读消息提示
}


// 发送文件
async function sendFile(file) {
    if (!window.currentChatPeer) return;
    
    const tauri = window.__TAURI__;
    
    if (tauri) {
        // 桌面端 - 不需要 file 参数，会自己弹出对话框
        console.log('[UI] 桌面端发送文件');
        
        try {
            const result = await apiSendFile(
                window.currentChatPeer.addr,
                null  // 桌面端不需要
            );
            
            // 从结果中获取文件信息
            const fileName = result.file_name || '未知文件';
            const fileSize = result.file_size || 0;
            
            // 显示发送的文件消息
            addMessageToChat({
                msg_type: 'file',
                from_id: 'me',
                content: fileName,
                file_name: fileName,
                file_size: fileSize,
                file_id: result.file_id,
                timestamp: Date.now() / 1000
            }, true);
            
            console.log('[UI] 文件发送成功');
        } catch (e) {
            console.error('[UI] 文件发送失败:', e);
            alert('文件发送失败: ' + e.message);
        }
    } else {
        // Web 端 - 使用传入的 file 参数
        console.log('[UI] Web 端发送文件:', file.name, file.size);
        
        try {
            const result = await apiSendFile(
                window.currentChatPeer.addr,
                file
            );
            
            // 显示发送的文件消息
            addMessageToChat({
                msg_type: 'file',
                from_id: 'me',
                content: file.name,
                file_name: file.name,
                file_size: file.size,
                file_id: result.file_id,
                timestamp: Date.now() / 1000
            }, true);
            
            console.log('[UI] 文件发送成功');
        } catch (e) {
            console.error('[UI] 文件发送失败:', e);
            alert('文件发送失败: ' + e.message);
        }
    }
}

// 格式化文件大小
function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

// 下载文件
async function downloadFile(fileId, fileName) {
    try {
        const url = `/api/download/${fileId}`;
        const a = document.createElement('a');
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        
        console.log('[UI] 开始下载文件:', fileName);
    } catch (e) {
        console.error('[UI] 下载文件失败:', e);
        alert('下载失败: ' + e.message);
    }
}
