// UI 交互逻辑

// 初始化改名功能
function initNameEditor() {
    const editBtn = document.getElementById('edit-name-btn');
    const editPanel = document.getElementById('edit-name-panel');
    const nameInput = document.getElementById('new-name-input');
    const saveBtn = document.getElementById('save-name-btn');
    const cancelBtn = document.getElementById('cancel-name-btn');
    const errorMsg = document.getElementById('error-msg');
    const nameDisplay = document.getElementById('my-name');

    // 点击编辑按钮 - 切换显示/隐藏
    editBtn.addEventListener('click', () => {
        if (editPanel.style.display === 'block') {
            // 当前是显示状态，点击后隐藏
            editPanel.style.display = 'none';
            errorMsg.textContent = '';
        } else {
            // 当前是隐藏状态，点击后显示
            editPanel.style.display = 'block';
            nameInput.value = '';
            nameInput.focus();
            errorMsg.textContent = '';
        }
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
        console.log('[UI] 文件状态:', message.file_status);
        
        const fileDiv = document.createElement('div');
        fileDiv.className = 'message-file';
        
        const fileStatus = message.file_status || 'accepted';
        const isPending = fileStatus === 'pending';
        const isAccepted = fileStatus === 'accepted';
        const isDownloading = fileStatus === 'downloading';
        const isUploading = fileStatus === 'uploading';
        
        fileDiv.innerHTML = `
            <div class="file-info">
                <span class="file-icon">📄</span>
                <div>
                    <div class="file-name">${message.file_name || message.content}</div>
                    <div class="file-size">${message.file_size ? formatFileSize(message.file_size) : '未知大小'}</div>
                    ${isAccepted && !isSent ? '<div class="file-finish">finish</div>' : ''}
                    ${isDownloading ? '<div class="file-downloading">下载中...</div>' : ''}
                    ${isUploading ? '<div class="file-uploading">上传中...</div>' : ''}
                </div>
            </div>
        `;
        
        // 如果是接收的文件
        if (!isSent && message.file_id) {
            if (isPending) {
                // 待接收状态 - 显示接收按钮
                const acceptBtn = document.createElement('button');
                acceptBtn.className = 'accept-file-btn';
                acceptBtn.textContent = '接收';
                acceptBtn.addEventListener('click', () => {
                    acceptFile(message.file_id, message.file_name || message.content);
                });
                fileDiv.appendChild(acceptBtn);
            } else if (isAccepted) {
                // 已接收状态 - 可以下载
                fileDiv.style.cursor = 'pointer';
                fileDiv.addEventListener('click', () => {
                    downloadFile(message.file_id, message.file_name || message.content);
                });
            }
            // isDownloading 和 isUploading 状态不添加任何交互
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
    console.log('[UI] ========== onReceiveMessage 被调用 ==========');
    console.log('[UI] 消息内容:', JSON.stringify(message, null, 2));
    console.log('[UI] 当前聊天对象:', window.currentChatPeer);
    
    // 如果正在和发送者聊天
    if (window.currentChatPeer && window.currentChatPeer.id === message.from_id) {
        console.log('[UI] ✓ 匹配当前聊天对象');
        
        // 检查是否是文件状态更新（downloading -> accepted/pending）
        if (message.msg_type === 'file' && message.file_status !== 'downloading') {
            // 刷新聊天历史以更新状态
            console.log('[UI] 文件状态更新 (' + message.file_status + ')，刷新聊天历史');
            loadChatHistory(window.currentChatPeer.id);
        } else {
            // 直接显示新消息
            console.log('[UI] 直接显示新消息 (msg_type=' + message.msg_type + ', file_status=' + message.file_status + ')');
            addMessageToChat(message, false);
        }
    } else {
        console.log('[UI] ✗ 不匹配当前聊天对象');
        console.log('[UI]   - message.from_id:', message.from_id);
        console.log('[UI]   - currentChatPeer.id:', window.currentChatPeer ? window.currentChatPeer.id : 'null');
    }
    
    console.log('[UI] ==========================================');
    
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
            // 先显示上传中的临时消息
            const tempFileId = 'temp_' + Date.now();
            addMessageToChat({
                msg_type: 'file',
                from_id: 'me',
                content: '准备发送...',
                file_name: '准备发送...',
                file_size: 0,
                file_id: tempFileId,
                file_status: 'uploading',
                timestamp: Date.now() / 1000
            }, true);
            
            const result = await apiSendFile(
                window.currentChatPeer.addr,
                null  // 桌面端不需要
            );
            
            // 上传完成，刷新聊天历史以显示正确的文件信息
            if (window.currentChatPeer) {
                await loadChatHistory(window.currentChatPeer.id);
            }
            
            console.log('[UI] 文件发送成功');
        } catch (e) {
            console.error('[UI] 文件发送失败:', e);
            alert('文件发送失败: ' + e.message);
            // 刷新聊天历史以移除失败的消息
            if (window.currentChatPeer) {
                await loadChatHistory(window.currentChatPeer.id);
            }
        }
    } else {
        // Web 端 - 使用传入的 file 参数
        console.log('[UI] ========== Web 端发送文件 ==========');
        console.log('[UI] 文件名:', file.name);
        console.log('[UI] 文件大小:', file.size);
        console.log('[UI] 目标地址:', window.currentChatPeer.addr);
        
        // 立即显示发送中的消息
        const tempFileId = 'temp_' + Date.now();
        const timestamp = Math.floor(Date.now() / 1000);
        
        console.log('[UI] 1. 在前端显示上传中消息');
        addMessageToChat({
            msg_type: 'file',
            from_id: 'me',
            content: file.name,
            file_name: file.name,
            file_size: file.size,
            file_id: tempFileId,
            file_status: 'uploading',  // 上传中状态
            timestamp: timestamp
        }, true);
        
        try {
            // 先在本地数据库创建上传记录
            console.log('[UI] 2. 调用 /api/create_upload_record');
            const createResp = await fetch('/api/create_upload_record', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    file_name: file.name,
                    timestamp: timestamp
                })
            });
            
            if (!createResp.ok) {
                throw new Error('创建上传记录失败: ' + createResp.status);
            }
            
            console.log('[UI] ✓ 上传记录已创建');
            
            console.log('[UI] 3. 开始上传文件到对方');
            const result = await apiSendFile(
                window.currentChatPeer.addr,
                file
            );
            
            console.log('[UI] ✓ 文件上传成功');
            
            // 上传成功，更新本地数据库状态为 'sent'
            console.log('[UI] 4. 更新上传状态为 sent');
            const updateResp = await fetch('/api/update_upload_status', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    file_name: file.name,
                    timestamp: timestamp,
                    status: 'sent'
                })
            });
            
            if (!updateResp.ok) {
                console.warn('[UI] ⚠ 更新上传状态失败:', updateResp.status);
            } else {
                console.log('[UI] ✓ 上传状态已更新');
            }
            
            // 刷新聊天历史以显示正确的状态
            console.log('[UI] 5. 刷新聊天历史');
            if (window.currentChatPeer) {
                await loadChatHistory(window.currentChatPeer.id);
            }
            
            console.log('[UI] ========== 文件发送完成 ==========');
        } catch (e) {
            console.error('[UI] ✗ 文件发送失败:', e);
            alert('文件发送失败: ' + e.message);
            // 删除失败的上传记录
            console.log('[UI] 删除失败的上传记录');
            await fetch('/api/delete_upload_record', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    file_name: file.name,
                    timestamp: timestamp
                })
            });
            // 刷新聊天历史以移除失败的消息
            if (window.currentChatPeer) {
                await loadChatHistory(window.currentChatPeer.id);
            }
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


// 初始化设置功能
function initSettings() {
    const settingsBtn = document.getElementById('settings-btn');
    const settingsPanel = document.getElementById('settings-panel');
    const saveSettingsBtn = document.getElementById('save-settings-btn');
    const cancelSettingsBtn = document.getElementById('cancel-settings-btn');
    const choosePathBtn = document.getElementById('choose-path-btn');
    const autoAcceptCheckbox = document.getElementById('auto-accept-checkbox');
    const downloadPathInput = document.getElementById('download-path-input');
    const settingsErrorMsg = document.getElementById('settings-error-msg');
    const settingsSuccessMsg = document.getElementById('settings-success-msg');
    
    // 打开/关闭设置面板 - 切换显示/隐藏
    settingsBtn.addEventListener('click', async () => {
        if (settingsPanel.style.display === 'block') {
            // 当前是显示状态，点击后隐藏
            settingsPanel.style.display = 'none';
            settingsErrorMsg.textContent = '';
            settingsSuccessMsg.textContent = '';
            settingsSuccessMsg.classList.remove('show');
        } else {
            // 当前是隐藏状态，点击后显示
            try {
                const settings = await apiGetSettings();
                autoAcceptCheckbox.checked = settings.auto_accept;
                downloadPathInput.value = settings.download_path;
                settingsPanel.style.display = 'block';
                settingsErrorMsg.textContent = '';
                settingsSuccessMsg.textContent = '';
                settingsSuccessMsg.classList.remove('show');
            } catch (e) {
                settingsErrorMsg.textContent = '加载设置失败: ' + e.message;
                settingsPanel.style.display = 'block';
            }
        }
    });
    
    // 选择路径
    choosePathBtn.addEventListener('click', async () => {
        const tauri = window.__TAURI__;
        if (tauri) {
            // 桌面端 - 使用 Tauri 对话框
            try {
                const selected = await tauri.dialog.open({
                    directory: true,
                    multiple: false,
                    title: '选择下载文件夹'
                });
                
                if (selected) {
                    const path = Array.isArray(selected) ? selected[0] : selected;
                    downloadPathInput.value = path;
                }
            } catch (e) {
                settingsErrorMsg.textContent = '选择路径失败: ' + e.message;
            }
        } else {
            // Web 端 - 只能手动输入
            const newPath = prompt('请输入下载路径:', downloadPathInput.value);
            if (newPath) {
                downloadPathInput.value = newPath;
            }
        }
    });
    
    // 保存设置
    saveSettingsBtn.addEventListener('click', async () => {
        try {
            settingsErrorMsg.textContent = '';
            settingsSuccessMsg.textContent = '';
            settingsSuccessMsg.classList.remove('show');
            
            await apiUpdateSettings(
                downloadPathInput.value,
                autoAcceptCheckbox.checked
            );
            
            // 显示成功消息
            settingsSuccessMsg.textContent = '✓ 设置保存成功';
            settingsSuccessMsg.classList.add('show');
            
            // 1.5秒后自动关闭设置面板
            setTimeout(() => {
                settingsPanel.style.display = 'none';
                settingsSuccessMsg.classList.remove('show');
            }, 1500);
            
            console.log('[UI] 设置保存成功');
        } catch (e) {
            settingsErrorMsg.textContent = '保存失败: ' + e.message;
        }
    });
    
    // 取消
    cancelSettingsBtn.addEventListener('click', () => {
        settingsPanel.style.display = 'none';
        settingsErrorMsg.textContent = '';
        settingsSuccessMsg.textContent = '';
        settingsSuccessMsg.classList.remove('show');
    });
}


// 接受文件
async function acceptFile(fileId, fileName) {
    console.log('[UI] ========== 开始接受文件 ==========');
    console.log('[UI] file_id:', fileId);
    console.log('[UI] file_name:', fileName);
    
    const tauri = window.__TAURI__;
    
    // 创建取消标志
    const cancelFlag = { cancelled: false };
    
    try {
        let savePath = null;
        
        if (tauri) {
            // 桌面端 - 弹出对话框选择保存位置
            console.log('[UI] 桌面端模式');
            const selected = await tauri.dialog.open({
                directory: true,
                multiple: false,
                title: '选择保存位置'
            });
            
            if (!selected) {
                console.log('[UI] 用户取消了选择');
                return;
            }
            
            savePath = Array.isArray(selected) ? selected[0] : selected;
            console.log('[UI] 选择的保存路径:', savePath);
            
            // 桌面端：调用 Tauri 命令，带重试逻辑
            await acceptFileWithRetry(tauri, fileId, savePath, fileName, cancelFlag);
        } else {
            // Web 端 - 直接使用默认路径
            console.log('[UI] Web 端模式，使用默认路径');
            console.log('[UI] 请求 URL:', `/api/accept_file/${fileId}`);
            
            // Web 端：调用 HTTP API，带重试逻辑
            await acceptFileHttpWithRetry(fileId, fileName, cancelFlag);
        }
        
        console.log('[UI] ==========================================');
    } catch (e) {
        if (e.message === 'USER_CANCELLED') {
            console.log('[UI] 用户取消了接收');
            return;
        }
        console.error('[UI] ✗ 接受文件失败:', e);
        console.error('[UI] 错误详情:', e.message);
        console.error('[UI] 错误堆栈:', e.stack);
        alert('接受文件失败: ' + e.message);
    }
}

// 桌面端接受文件（带重试和取消）
async function acceptFileWithRetry(tauri, fileId, savePath, fileName, cancelFlag) {
    let retryCount = 0;
    
    while (!cancelFlag.cancelled) {
        try {
            await tauri.core.invoke('accept_file', {
                fileId,
                savePath
            });
            
            console.log('[UI] ✓ 文件接收成功');
            
            // 刷新聊天历史
            if (window.currentChatPeer) {
                console.log('[UI] 刷新聊天历史...');
                await loadChatHistory(window.currentChatPeer.id);
                console.log('[UI] 聊天历史已刷新');
            }
            
            return; // 成功，退出
        } catch (e) {
            if (e.includes('还在下载中') || e.includes('下载中')) {
                retryCount++;
                console.log(`[UI] 文件下载中，等待... (第 ${retryCount} 次重试)`);
                
                // 第一次显示下载中状态
                if (retryCount === 1 && window.currentChatPeer) {
                    await loadChatHistory(window.currentChatPeer.id);
                    // 显示取消按钮
                    showCancelButton(fileId, cancelFlag);
                }
                
                await new Promise(resolve => setTimeout(resolve, 2000)); // 等待2秒
                continue;
            }
            throw e; // 其他错误，直接抛出
        }
    }
    
    throw new Error('USER_CANCELLED');
}

// Web 端接受文件（带重试和取消）
async function acceptFileHttpWithRetry(fileId, fileName, cancelFlag) {
    let retryCount = 0;
    
    while (!cancelFlag.cancelled) {
        const resp = await fetch(`/api/accept_file/${fileId}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ save_path: null })
        });
        
        console.log('[UI] API 响应状态:', resp.status, resp.statusText);
        
        if (resp.status === 202) {
            // 202 表示文件还在下载中
            retryCount++;
            console.log(`[UI] 文件下载中，等待... (第 ${retryCount} 次重试)`);
            
            // 第一次显示下载中状态
            if (retryCount === 1 && window.currentChatPeer) {
                await loadChatHistory(window.currentChatPeer.id);
                // 显示取消按钮
                showCancelButton(fileId, cancelFlag);
            }
            
            await new Promise(resolve => setTimeout(resolve, 2000)); // 等待2秒
            continue;
        }
        
        if (!resp.ok) {
            const errorText = await resp.text();
            console.error('[UI] ✗ API 错误响应:', errorText);
            throw new Error('接受文件失败: HTTP ' + resp.status + ' - ' + errorText);
        }
        
        const result = await resp.json();
        console.log('[UI] ✓ API 响应成功:', result);
        
        // 隐藏取消按钮
        hideCancelButton(fileId);
        
        // 刷新聊天历史
        if (window.currentChatPeer) {
            console.log('[UI] 刷新聊天历史...');
            await loadChatHistory(window.currentChatPeer.id);
            console.log('[UI] 聊天历史已刷新');
        }
        
        return; // 成功，退出
    }
    
    throw new Error('USER_CANCELLED');
}

// 显示取消按钮
function showCancelButton(fileId, cancelFlag) {
    const chatMessages = document.getElementById('chat-messages');
    const messages = chatMessages.querySelectorAll('.message');
    
    for (const messageDiv of messages) {
        const fileDiv = messageDiv.querySelector('.message-file');
        if (!fileDiv) continue;
        
        // 检查是否是当前文件（通过文件名或其他标识）
        const downloadingDiv = fileDiv.querySelector('.file-downloading');
        if (downloadingDiv) {
            // 检查是否已经有取消按钮
            if (!fileDiv.querySelector('.cancel-download-btn')) {
                const cancelBtn = document.createElement('button');
                cancelBtn.className = 'cancel-download-btn';
                cancelBtn.textContent = '取消';
                cancelBtn.dataset.fileId = fileId;
                cancelBtn.addEventListener('click', () => {
                    cancelFlag.cancelled = true;
                    hideCancelButton(fileId);
                    console.log('[UI] 用户取消了下载');
                });
                fileDiv.appendChild(cancelBtn);
            }
        }
    }
}

// 隐藏取消按钮
function hideCancelButton(fileId) {
    const cancelBtns = document.querySelectorAll('.cancel-download-btn');
    for (const btn of cancelBtns) {
        if (btn.dataset.fileId === fileId) {
            btn.remove();
        }
    }
}
