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
			nameDisplay.textContent = updatedName;
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
	li.dataset.name = name;
	li.dataset.addr = addr;
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
// 更新用户状态 - 赛博神医微创版
function updateUserStatus(item, name, addr, isOffline) {
	const statusSpan = item.querySelector('.user-status');
	const nameSpan = item.querySelector('.user-name');
	const addrSpan = item.querySelector('.user-addr');

	// 1. 更新基础信息
	if (nameSpan) nameSpan.textContent = name;
	if (addrSpan) addrSpan.textContent = addr;

	// 2. 更新状态标签的文字
	if (statusSpan) {
		// 离线显示 OFFLINE，在线清空
		statusSpan.textContent = isOffline ? 'OFF' : '';
	}

	// 3. 类名手术：使用你的原有逻辑，但确保 CSS 能跟上
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
	const chatContainer = document.getElementById('chat-container');

	// 关闭聊天窗口
	closeChatBtn.addEventListener('click', () => {
		closeChat();
	});

	// 发送消息
	sendBtn.addEventListener('click', () => {
		sendMessage();
	});

	// 自动调整 textarea 高度
	function adjustTextareaHeight() {
		chatInput.style.height = 'auto';
		const newHeight = Math.min(chatInput.scrollHeight, 200);
		chatInput.style.height = newHeight + 'px';
	}

	// 输入时调整高度
	chatInput.addEventListener('input', adjustTextareaHeight);

	// 回车发送（Shift+Enter 换行）
	chatInput.addEventListener('keypress', (e) => {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
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

	// 拖拽文件功能
	initDragAndDrop(chatContainer);

	// 粘贴文件功能
	initPasteFile();
}

// --- 赛博加固版 JS ---

// 1. 打开聊天
function openChat(peer) {
	// [检查点]：如果已经是当前聊天的用户，且窗口开着，就别折腾了
	const chatContainer = document.getElementById('chat-container');
	if (window.currentChatPeer && window.currentChatPeer.id === peer.id && chatContainer.style.display === 'flex') {
		return;
	}

	// [关键修复]：如果是在手机端，确保 Hash 状态同步
	if (window.innerWidth <= 768) {
		if (window.location.hash !== '#chat') {
			window.history.pushState({ chatOpen: true }, "", "#chat");
		}
	}

	window.currentChatPeer = peer;

	const chatWithName = document.getElementById('chat-with-name');
	const chatMessages = document.getElementById('chat-messages');

	chatContainer.style.display = 'flex';
	chatWithName.textContent = `${peer.name}`;
	chatMessages.innerHTML = '';

	// 高亮逻辑
	updateListHighlight(peer.id);

	window.lastMessageTimestamp = 0;
	loadChatHistory(peer.id);
	console.log('[UI] 成功进入聊天:', peer.name);
}

// 2. 关闭聊天（由 X 按钮或物理返回键调用）
function closeChat() {
	// 如果是手机端且有 #chat，点击 X 按钮时触发 back() 即可，剩下的交给 popstate
	if (window.innerWidth <= 768 && window.location.hash === '#chat') {
		window.history.back();
		return;
	}
	performCloseChatUI();
}

// 3. 真正的 UI 隐藏逻辑（只管藏，不管历史记录）
function performCloseChatUI() {
	const chatContainer = document.getElementById('chat-container');
	if (chatContainer) chatContainer.style.display = 'none';
	window.currentChatPeer = null;
	updateListHighlight(null); // 清除高亮
}

// 4. 辅助函数：更新高亮
function updateListHighlight(activeId) {
	const items = document.querySelectorAll('#user-list li');
	items.forEach(item => {
		if (activeId && item.dataset.id === activeId) {
			item.classList.add('active');
		} else {
			item.classList.remove('active');
		}
	});
}

// 5. [最关键的手术] 全局监听器：处理物理返回键和手动后退
window.addEventListener('popstate', function(event) {
	const chatContainer = document.getElementById('chat-container');
	// 如果检测到 URL 里没有 #chat 了，但窗口还开着，强制关掉它
	if (window.location.hash !== '#chat') {
		performCloseChatUI();
	}
});

// 发送消息
async function sendMessage() {
	if (!window.currentChatPeer) return;

	const chatInput = document.getElementById('chat-input');
	const content = chatInput.value.trim();

	if (!content) return;

	try {
		// 调用 API 发送消息
		await apiSendMessage(window.currentChatPeer.id, window.currentChatPeer.addr, content);

		// 清空输入框并重置高度
		chatInput.value = '';
		chatInput.style.height = 'auto';

		// 显示消息
		addMessageToChat({
			from_id: 'me',
			content: content,
			timestamp: Date.now() / 1000
		}, true);

		// 发送消息后滚动到底部
		const chatMessages = document.getElementById('chat-messages');
		chatMessages.scrollTop = chatMessages.scrollHeight;

		console.log('[UI] 发送消息:', content);
	} catch (e) {
		console.error('[UI] 发送消息失败:', e);
		alert('发送失败: ' + e.message);
	}
}

// 添加消息到聊天窗口
function addMessageToChat(message, isSent) {
    const chatMessages = document.getElementById('chat-messages');
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${isSent ? 'sent' : 'received'}`;

    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';

    if (message.msg_type === 'file') {
        const fileContainer = document.createElement('div');
        fileContainer.className = 'message-file';

        // 检查是否是图片文件
        const isImage = isImageFile(message.file_name || message.content);

        if (isImage && message.file_path && (message.file_status === 'sent' || message.file_status === 'accepted')) {
            // 图片预览
            const imgPreview = document.createElement('div');
            imgPreview.className = 'image-preview';

            const img = document.createElement('img');
            
            const tauri = window.__TAURI__;
            if (tauri) {
                // 桌面端：使用 convertFileSrc
                const assetUrl = tauri.core.convertFileSrc(message.file_path);
                img.src = assetUrl;
            } else {
                // Web 端：使用下载 API
                if (message.file_id) {
                    img.src = `/api/download/${message.file_id}`;
                }
            }

            img.alt = message.file_name || message.content;
            img.loading = 'lazy';
            
            // 图片加载失败时显示文件图标
            img.onerror = () => {
                imgPreview.innerHTML = '';
                imgPreview.appendChild(createFileIcon(message));
            };

            imgPreview.appendChild(img);
            fileContainer.appendChild(imgPreview);
        } else {
            // 非图片或未完成的文件：显示文件图标
            fileContainer.appendChild(createFileIcon(message));
        }

        contentDiv.appendChild(fileContainer);

        // 3. 状态标签 (保留类名，供默认模式显示圆点，伪装模式显示注释)
        const fileStatus = message.file_status || 'accepted';
        const statusDiv = document.createElement('div');
        if (fileStatus === 'downloading') {
            statusDiv.className = 'file-downloading';
            const speed = message.transfer_speed ? Math.round(message.transfer_speed) + ' MB/s' : '下载中...';
            statusDiv.textContent = speed;
        } else if (fileStatus === 'uploading') {
            statusDiv.className = 'file-uploading';
            const speed = message.transfer_speed ? Math.round(message.transfer_speed) + ' MB/s' : '上传中...';
            statusDiv.textContent = speed;
        } else if (fileStatus === 'accepted' && !isSent) {
            statusDiv.className = 'file-finish';
            statusDiv.textContent = 'finish';
        }

        if (statusDiv.className) {
            contentDiv.appendChild(statusDiv);
        }

        // 对于已完成的文件（sent 或 accepted），添加点击处理
        if (fileStatus === 'sent' || fileStatus === 'accepted') {
            fileContainer.style.cursor = 'pointer';
            
            const tauri = window.__TAURI__;
            if (tauri) {
                // 桌面端/Android：点击打开文件所在位置或分享
                if (message.file_path) {
                    // 检查是否是 Android
                    const isAndroid = navigator.userAgent.includes('Android');
                    
                    if (isAndroid) {
                        // Android 端：所有文件都支持分享到其他应用
                        fileContainer.addEventListener('click', async () => {
                            try {
                                await apiShareFileToOtherApp(message.file_path);
                            } catch (e) {
                                alert('分享失败: ' + e.message);
                            }
                        });
                    } else {
                        // 桌面端：点击打开文件位置
                        fileContainer.addEventListener('click', () => openFileLocation(message.file_path));
                    }
                }
            } else {
                // Web 端：点击下载文件
                if (message.file_id) {
                    fileContainer.addEventListener('click', () => downloadFile(message.file_id, message.file_name || message.content));
                }
            }
        }
    } else {
        const textSpan = document.createElement('span');
        textSpan.className = 'message-text';
        textSpan.textContent = message.content;
        contentDiv.appendChild(textSpan);
    }

    const timeDiv = document.createElement('div');
    timeDiv.className = 'message-time';
    const date = new Date(message.timestamp * 1000);
    timeDiv.textContent = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

    messageDiv.appendChild(contentDiv);
    messageDiv.appendChild(timeDiv);
    chatMessages.appendChild(messageDiv);
}

// 创建文件图标元素
function createFileIcon(message) {
    const fileInfo = document.createElement('div');
    fileInfo.className = 'file-info-wrapper';

    // 1. 图标
    const fileIcon = document.createElement('span');
    fileIcon.className = 'file-icon';
    fileIcon.textContent = '📄';

    // 2. 文件信息
    const fileInfoText = document.createElement('div');
    fileInfoText.className = 'file-info';

    // 文件名
    const fileName = document.createElement('div');
    fileName.className = 'file-name';
    fileName.textContent = message.file_name || message.content;

    // 文件大小
    const fileSize = document.createElement('div');
    fileSize.className = 'file-size';
    fileSize.textContent = message.file_size ? formatFileSize(message.file_size) : '未知大小';

    fileInfoText.appendChild(fileName);
    fileInfoText.appendChild(fileSize);
    
    fileInfo.appendChild(fileIcon);
    fileInfo.appendChild(fileInfoText);

    return fileInfo;
}

// 检查是否是图片文件
function isImageFile(fileName) {
    if (!fileName) return false;
    
    const imageExtensions = ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.svg', '.ico'];
    const lowerFileName = fileName.toLowerCase();
    
    return imageExtensions.some(ext => lowerFileName.endsWith(ext));
}

// 加载聊天历史
async function loadChatHistory(peerId, preserveScroll = false) {
	try {
		const messages = await apiGetChatHistory(peerId);

		const chatMessages = document.getElementById('chat-messages');

		// 保存当前滚动位置
		const oldScrollTop = chatMessages.scrollTop;
		const oldScrollHeight = chatMessages.scrollHeight;
		const wasAtBottom = oldScrollHeight - oldScrollTop - chatMessages.clientHeight < 100;

		chatMessages.innerHTML = '';

		for (const msg of messages) {
			addMessageToChat(msg, msg.from_id === 'me');
			// 更新最后消息时间戳
			if (msg.timestamp > (window.lastMessageTimestamp || 0)) {
				window.lastMessageTimestamp = msg.timestamp;
			}
		}

		// 恢复滚动位置
		if (preserveScroll && !wasAtBottom) {
			// 如果用户不在底部，尝试保持相对位置
			const newScrollHeight = chatMessages.scrollHeight;
			const scrollDiff = newScrollHeight - oldScrollHeight;
			chatMessages.scrollTop = oldScrollTop + scrollDiff;
		} else if (!preserveScroll || wasAtBottom) {
			// 首次加载或用户在底部时，滚动到底部
			chatMessages.scrollTop = chatMessages.scrollHeight;
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
			// 刷新聊天历史以更新状态，保持滚动位置
			console.log('[UI] 文件状态更新 (' + message.file_status + ')，刷新聊天历史');
			loadChatHistory(window.currentChatPeer.id, true);
		} else {
			// 直接显示新消息
			console.log('[UI] 直接显示新消息 (msg_type=' + message.msg_type + ', file_status=' + message.file_status + ')');

			const chatMessages = document.getElementById('chat-messages');
			const wasAtBottom = chatMessages.scrollHeight - chatMessages.scrollTop - chatMessages.clientHeight < 100;

			addMessageToChat(message, false);

			// 只有在底部时才滚动
			if (wasAtBottom) {
				chatMessages.scrollTop = chatMessages.scrollHeight;
			}
		}
	} else {
		console.log('[UI] ✗ 不匹配当前聊天对象');
		console.log('[UI]   - message.from_id:', message.from_id);
		console.log('[UI]   - currentChatPeer.id:', window.currentChatPeer ? window.currentChatPeer.id : 'null');
	}

	console.log('[UI] ==========================================');

	// TODO: 显示未读消息提示
}


// 通过文件路径发送文件（桌面端零拷贝，直接从硬盘读取）
async function sendFileByPath(filePath) {
	if (!window.currentChatPeer) return;

	const tauri = window.__TAURI__;
	
	if (!tauri) {
		console.error('[UI] sendFileByPath 只能在桌面端使用');
		return;
	}

	console.log('[UI] 通过路径发送文件（零拷贝）:', filePath);

	try {
		// 处理 file:// URI 格式
		let actualPath = filePath;
		if (filePath.startsWith('file://')) {
			// 移除 file:// 前缀并解码 URL 编码
			actualPath = decodeURIComponent(filePath.substring(7));
			console.log('[UI] 转换 URI 为路径:', actualPath);
		}

		// 获取文件名
		const fileName = actualPath.split(/[\\/]/).pop();
		
		// 获取文件大小
		let fileSize = 0;
		try {
			const metadata = await tauri.fs.stat(actualPath);
			fileSize = metadata.size;
		} catch (e) {
			console.warn('[UI] 无法获取文件大小:', e);
		}

		// 显示上传中的临时消息
		const tempFileId = 'temp_' + Date.now();
		addMessageToChat({
			msg_type: 'file',
			from_id: 'me',
			content: fileName,
			file_name: fileName,
			file_size: fileSize,
			file_id: tempFileId,
			file_status: 'uploading',
			timestamp: Date.now() / 1000
		}, true);

		// 直接调用 send_file 命令，传递文件路径
		// Rust 会直接从硬盘读取文件，零拷贝
		const result = await apiSendFile(
			window.currentChatPeer.id,
			window.currentChatPeer.addr,
			null,
			actualPath
		);

		// 上传完成，刷新聊天历史
		if (window.currentChatPeer) {
			await loadChatHistory(window.currentChatPeer.id);
		}

		console.log('[UI] 文件发送成功（零拷贝）');
	} catch (e) {
		console.error('[UI] 文件发送失败:', e);
		alert('文件发送失败: ' + e.message);
		// 刷新聊天历史以移除失败的消息
		if (window.currentChatPeer) {
			await loadChatHistory(window.currentChatPeer.id);
		}
	}
}

// 发送文件
async function sendFile(file) {
	if (!window.currentChatPeer) return;

	const tauri = window.__TAURI__;

	if (tauri) {
		// 桌面端
		console.log('[UI] 桌面端发送文件');

		// 如果传入了 file 参数（拖拽或粘贴），需要特殊处理
		if (file) {
			console.log('[UI] 处理拖拽/粘贴的文件:', file.name, file.size);
			
			// 桌面端拖拽/粘贴时，我们需要先将文件保存到临时目录
			// 然后再调用 send_file 命令
			try {
				// 显示上传中的临时消息
				const tempFileId = 'temp_' + Date.now();
				addMessageToChat({
					msg_type: 'file',
					from_id: 'me',
					content: file.name,
					file_name: file.name,
					file_size: file.size,
					file_id: tempFileId,
					file_status: 'uploading',
					timestamp: Date.now() / 1000
				}, true);

				// 读取文件内容
				const arrayBuffer = await file.arrayBuffer();
				const uint8Array = new Uint8Array(arrayBuffer);

				// 获取临时目录路径
				const tempDir = await tauri.path.tempDir();
				const tempFilePath = await tauri.path.join(tempDir, file.name);

				// 写入临时文件
				await tauri.fs.writeFile(tempFilePath, uint8Array);
				console.log('[UI] 文件已保存到临时目录:', tempFilePath);

				// 调用 send_file 命令
				const result = await apiSendFile(
					window.currentChatPeer.id,
					window.currentChatPeer.addr,
					null,
					tempFilePath  // 传递临时文件路径
				);

				// 上传完成，刷新聊天历史
				if (window.currentChatPeer) {
					await loadChatHistory(window.currentChatPeer.id);
				}

				// 删除临时文件
				try {
					await tauri.fs.remove(tempFilePath);
					console.log('[UI] 临时文件已删除');
				} catch (e) {
					console.warn('[UI] 删除临时文件失败:', e);
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
			// 没有传入 file 参数，使用文件对话框选择
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
					window.currentChatPeer.id,
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
					timestamp: timestamp,
					receiver_id: window.currentChatPeer.id  // 添加接收者ID
				})
			});

			if (!createResp.ok) {
				throw new Error('创建上传记录失败: ' + createResp.status);
			}

			console.log('[UI] ✓ 上传记录已创建');

			console.log('[UI] 3. 开始上传文件到对方');
			const result = await apiSendFile(
				window.currentChatPeer.id,
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
		
		// 使用 fetch 来获取下载进度
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`HTTP ${response.status}`);
		}
		
		const contentLength = response.headers.get('content-length');
		const totalSize = parseInt(contentLength, 10);
		
		const reader = response.body.getReader();
		const chunks = [];
		let receivedLength = 0;
		const startTime = Date.now();
		let lastLogTime = startTime;
		
		// 更新下载速度显示
		const updateDownloadSpeed = () => {
			const elapsed = (Date.now() - startTime) / 1000;
			if (elapsed > 0) {
				const speed = receivedLength / (1024 * 1024) / elapsed;
				const statusDivs = document.querySelectorAll('.file-downloading');
				statusDivs.forEach(div => {
					div.textContent = Math.round(speed) + ' MB/s';
				});
			}
		};
		
		while (true) {
			const {done, value} = await reader.read();
			
			if (done) break;
			
			chunks.push(value);
			receivedLength += value.length;
			
			// 每秒更新一次速度显示
			const now = Date.now();
			if (now - lastLogTime > 1000) {
				updateDownloadSpeed();
				lastLogTime = now;
			}
		}
		
		// 合并所有分块
		const chunksAll = new Uint8Array(receivedLength);
		let position = 0;
		for (const chunk of chunks) {
			chunksAll.set(chunk, position);
			position += chunk.length;
		}
		
		// 创建 Blob 并下载
		const blob = new Blob([chunksAll]);
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = fileName;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(a.href);
		
		const totalTime = (Date.now() - startTime) / 1000;
		const avgSpeed = (receivedLength / (1024 * 1024)) / totalTime;
		console.log('[UI] ✓ 文件下载完成:', fileName, '耗时:', totalTime.toFixed(2), '秒，平均速度:', avgSpeed.toFixed(2), 'MB/s');
	} catch (e) {
		console.error('[UI] 下载文件失败:', e);
		alert('下载失败: ' + e.message);
	}
}

// 打开文件所在位置（仅桌面端）
async function openFileLocation(filePath) {
	const tauri = window.__TAURI__;
	
	if (!tauri) {
		alert('此功能仅在桌面端支持');
		return;
	}
	
	try {
		await tauri.core.invoke('open_file_location', { filePath: filePath });
		console.log('[UI] ✓ 打开文件位置:', filePath);
	} catch (e) {
		console.error('[UI] 打开文件位置失败:', e);
		alert('打开文件位置失败: ' + e.message);
	}
}


// 初始化设置功能
function initSettings() {
	const settingsBtn = document.getElementById('settings-btn');
	const settingsPanel = document.getElementById('settings-panel');
	const saveSettingsBtn = document.getElementById('save-settings-btn');
	const cancelSettingsBtn = document.getElementById('cancel-settings-btn');
	const choosePathBtn = document.getElementById('choose-path-btn');
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
		const isAndroid = tauri && navigator.userAgent.includes('Android');

		if (isAndroid) {
			// Android - 显示路径选择面板
			const androidPathPanel = document.getElementById('android-path-panel');
			androidPathPanel.style.display = 'block';
		} else if (tauri) {
			// 桌面端 - 使用 Tauri 对话框
			try {
				const defaultPath = await apiGetDefaultDownloadPath();
				const selected = await tauri.dialog.open({
					directory: true,
					multiple: false,
					title: '选择下载文件夹',
					defaultPath: downloadPathInput.value || defaultPath
				});

				if (selected) {
					const path = Array.isArray(selected) ? selected[0] : selected;
					downloadPathInput.value = path;
					settingsErrorMsg.textContent = '';
				}
			} catch (e) {
				console.error('[UI] 文件选择器错误:', e);
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

	// Android 路径选择面板逻辑
	const androidPathPanel = document.getElementById('android-path-panel');
	const pathOptions = document.querySelectorAll('.path-option');
	const customPathInput = document.getElementById('custom-path-input');
	const useCustomPathBtn = document.getElementById('use-custom-path-btn');
	const cancelAndroidPathBtn = document.getElementById('cancel-android-path-btn');

	pathOptions.forEach(option => {
		option.addEventListener('click', () => {
			const path = option.getAttribute('data-path');
			downloadPathInput.value = path;
			androidPathPanel.style.display = 'none';
		});
	});

	useCustomPathBtn.addEventListener('click', () => {
		const customPath = customPathInput.value.trim();
		if (customPath) {
			downloadPathInput.value = customPath;
			androidPathPanel.style.display = 'none';
			customPathInput.value = '';
		}
	});

	cancelAndroidPathBtn.addEventListener('click', () => {
		androidPathPanel.style.display = 'none';
		customPathInput.value = '';
	});

	// 保存设置
	saveSettingsBtn.addEventListener('click', async () => {
		try {
			settingsErrorMsg.textContent = '';
			settingsSuccessMsg.textContent = '';
			settingsSuccessMsg.classList.remove('show');

			await apiUpdateSettings(
				downloadPathInput.value
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






// 初始化主题功能
function initTheme() {
	const themeBtn = document.getElementById('theme-btn');
	const themePanel = document.getElementById('theme-panel');
	const applyThemeBtn = document.getElementById('apply-theme-btn');
	const cancelThemeBtn = document.getElementById('cancel-theme-btn');
	const themeList = document.getElementById('theme-list');
	const themeErrorMsg = document.getElementById('theme-error-msg');
	const themeSuccessMsg = document.getElementById('theme-success-msg');

	// 打开/关闭主题面板
	themeBtn.addEventListener('click', async () => {
		if (themePanel.style.display === 'block') {
			themePanel.style.display = 'none';
			themeErrorMsg.textContent = '';
			themeSuccessMsg.textContent = '';
			themeSuccessMsg.classList.remove('show');
		} else {
			try {
				await loadThemeList();
				themePanel.style.display = 'block';
				themeErrorMsg.textContent = '';
				themeSuccessMsg.textContent = '';
				themeSuccessMsg.classList.remove('show');
			} catch (e) {
				themeErrorMsg.textContent = '加载主题列表失败: ' + e.message;
				themePanel.style.display = 'block';
			}
		}
	});

	// 应用主题
	applyThemeBtn.addEventListener('click', async () => {
		const selectedTheme = document.querySelector('input[name="theme"]:checked');
		if (!selectedTheme) {
			themeErrorMsg.textContent = '请选择一个主题';
			return;
		}

		try {
			themeErrorMsg.textContent = '';
			themeSuccessMsg.textContent = '';
			themeSuccessMsg.classList.remove('show');

			await applyTheme(selectedTheme.value);
			await apiSaveCurrentTheme(selectedTheme.value);

			themeSuccessMsg.textContent = '✓ 主题应用成功';
			themeSuccessMsg.classList.add('show');

			setTimeout(() => {
				themePanel.style.display = 'none';
				themeSuccessMsg.classList.remove('show');
			}, 1500);

			console.log('[UI] 主题应用成功:', selectedTheme.value);
		} catch (e) {
			themeErrorMsg.textContent = '应用主题失败: ' + e.message;
			console.error('[UI] 应用主题失败:', e);
		}
	});

	// 取消
	cancelThemeBtn.addEventListener('click', () => {
		themePanel.style.display = 'none';
		themeErrorMsg.textContent = '';
		themeSuccessMsg.textContent = '';
		themeSuccessMsg.classList.remove('show');
	});

	// 页面加载时应用保存的主题
	loadSavedTheme();
}

// 加载主题列表
async function loadThemeList() {
	const themeList = document.getElementById('theme-list');
	const themes = await apiGetThemeList();
	const currentTheme = await apiGetCurrentTheme();

	themeList.innerHTML = '';

	for (const theme of themes) {
		const themeItem = document.createElement('div');
		themeItem.className = 'theme-item';

		const isSelected = theme.name === currentTheme;

		themeItem.innerHTML = `
            <input type="radio" id="theme-${theme.name}" name="theme" value="${theme.name}" ${isSelected ? 'checked' : ''}>
            <label for="theme-${theme.name}">${theme.display_name}${theme.is_custom ? ' (自定义)' : ''}</label>
        `;

		if (isSelected) {
			themeItem.classList.add('active');
		}

		// 点击整个项目也能选中
		themeItem.addEventListener('click', (e) => {
			if (e.target.tagName !== 'INPUT') {
				const radio = themeItem.querySelector('input[type="radio"]');
				radio.checked = true;

				// 更新active状态
				document.querySelectorAll('.theme-item').forEach(item => item.classList.remove('active'));
				themeItem.classList.add('active');
			}
		});

		// 监听radio变化
		const radio = themeItem.querySelector('input[type="radio"]');
		radio.addEventListener('change', () => {
			if (radio.checked) {
				document.querySelectorAll('.theme-item').forEach(item => item.classList.remove('active'));
				themeItem.classList.add('active');
			}
		});

		themeList.appendChild(themeItem);
	}

	console.log('[UI] 加载了', themes.length, '个主题，当前主题:', currentTheme);
}

// 应用主题
async function applyTheme(themeName) {
	// 移除现有的自定义主题样式
	const existingCustomStyle = document.getElementById('custom-theme-style');
	if (existingCustomStyle) {
		existingCustomStyle.remove();
	}

	// 获取默认样式表
	const defaultStylesheet = document.querySelector('link[href="css/style.css"]');

	if (themeName === 'default') {
		// 恢复默认主题：启用默认CSS
		if (defaultStylesheet) {
			defaultStylesheet.disabled = false;
		}
		console.log('[UI] 应用默认主题');
		return;
	}

	// 获取自定义主题CSS
	const css = await apiGetThemeCss(themeName);

	// 禁用默认样式表
	if (defaultStylesheet) {
		defaultStylesheet.disabled = true;
	}

	// 创建新的style元素
	const styleElement = document.createElement('style');
	styleElement.id = 'custom-theme-style';
	styleElement.textContent = css;

	// 添加到head中
	document.head.appendChild(styleElement);

	console.log('[UI] 应用自定义主题:', themeName, '(已禁用默认CSS)');
}

// 加载保存的主题
async function loadSavedTheme() {
	try {
		const currentTheme = await apiGetCurrentTheme();
		if (currentTheme && currentTheme !== 'default') {
			await applyTheme(currentTheme);
			console.log('[UI] 自动加载保存的主题:', currentTheme);
		}
	} catch (e) {
		console.warn('[UI] 加载保存的主题失败:', e);
	}
}

// 初始化拖拽文件功能
function initDragAndDrop(chatContainer) {
	console.log('[UI] 初始化拖拽文件功能');

	const tauri = window.__TAURI__;

	if (tauri) {
		// 桌面端：使用 Tauri 的原生拖拽事件（可以获取文件路径）
		console.log('[UI] 使用 Tauri 原生拖拽事件');
		
		// 监听 Tauri 的文件拖放事件
		tauri.event.listen('tauri://drag-drop', async (event) => {
			console.log('[UI] Tauri 拖放事件:', event);
			
			if (!window.currentChatPeer) {
				console.log('[UI] 没有打开聊天窗口，忽略拖放');
				return;
			}

			const paths = event.payload.paths;
			if (paths && paths.length > 0) {
				console.log('[UI] 拖放的文件路径:', paths);
				
				// 依次发送所有文件（使用文件路径，零拷贝）
				for (const filePath of paths) {
					console.log('[UI] 发送文件:', filePath);
					await sendFileByPath(filePath);
				}
			}
		});

		// 监听拖拽悬停事件（显示视觉反馈）
		tauri.event.listen('tauri://drag-enter', () => {
			if (window.currentChatPeer) {
				chatContainer.classList.add('drag-over');
			}
		});

		tauri.event.listen('tauri://drag-leave', () => {
			chatContainer.classList.remove('drag-over');
		});

		tauri.event.listen('tauri://drag-drop', () => {
			chatContainer.classList.remove('drag-over');
		});

	} else {
		// Web 端：使用传统的 HTML5 拖拽 API（需要读取文件内容）
		console.log('[UI] 使用 HTML5 拖拽 API');

		// 防止默认的拖拽行为（打开文件）
		['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
			chatContainer.addEventListener(eventName, preventDefaults, false);
			document.body.addEventListener(eventName, preventDefaults, false);
		});

		function preventDefaults(e) {
			e.preventDefault();
			e.stopPropagation();
		}

		// 拖拽进入时高亮
		['dragenter', 'dragover'].forEach(eventName => {
			chatContainer.addEventListener(eventName, () => {
				if (window.currentChatPeer) {
					chatContainer.classList.add('drag-over');
				}
			}, false);
		});

		// 拖拽离开时取消高亮
		['dragleave', 'drop'].forEach(eventName => {
			chatContainer.addEventListener(eventName, () => {
				chatContainer.classList.remove('drag-over');
			}, false);
		});

		// 处理文件拖放
		chatContainer.addEventListener('drop', async (e) => {
			if (!window.currentChatPeer) {
				console.log('[UI] 没有打开聊天窗口，忽略拖放');
				return;
			}

			const files = e.dataTransfer.files;
			
			if (files && files.length > 0) {
				console.log('[UI] 拖放了', files.length, '个文件');
				
				// 依次发送所有文件
				for (let i = 0; i < files.length; i++) {
					const file = files[i];
					console.log('[UI] 拖放的文件:', file.name, file.size);
					await sendFile(file);
				}
			} else {
				console.log('[UI] 没有检测到文件');
			}
		}, false);
	}
}

// 初始化粘贴文件功能
function initPasteFile() {
	console.log('[UI] 初始化粘贴文件功能');

	const tauri = window.__TAURI__;

	// 监听全局粘贴事件
	document.addEventListener('paste', async (e) => {
		// 只在聊天窗口打开时处理
		if (!window.currentChatPeer) {
			console.log('[UI] 没有打开聊天窗口，忽略粘贴');
			return;
		}

		// 桌面端：优先尝试使用 clipboard-rs 读取文件路径（零拷贝）
		if (tauri) {
			try {
				console.log('[UI] 尝试从剪贴板读取文件路径');
				const filePaths = await tauri.core.invoke('read_clipboard_files');
				
				if (filePaths && filePaths.length > 0) {
					console.log('[UI] 剪贴板中的文件路径:', filePaths);
					e.preventDefault(); // 阻止默认粘贴行为
					
					// 使用零拷贝方式发送文件
					for (const filePath of filePaths) {
						await sendFileByPath(filePath);
					}
					return;
				} else {
					console.log('[UI] 剪贴板中没有文件');
				}
			} catch (err) {
				console.log('[UI] 读取剪贴板文件路径失败，尝试使用传统方式:', err);
				// 继续使用传统方式处理
			}
		}

		// 传统方式：从 ClipboardEvent 读取文件（需要读取内容）
		const clipboardData = e.clipboardData || window.clipboardData;
		if (!clipboardData) {
			console.log('[UI] 无法访问剪贴板');
			return;
		}

		// 检查是否有文件
		const items = clipboardData.items;
		if (!items || items.length === 0) {
			console.log('[UI] 剪贴板中没有内容');
			return;
		}

		let hasFile = false;

		for (let i = 0; i < items.length; i++) {
			const item = items[i];
			console.log('[UI] 剪贴板项目类型:', item.type, item.kind);

			if (item.kind === 'file') {
				hasFile = true;
				e.preventDefault(); // 阻止默认粘贴行为

				const file = item.getAsFile();
				if (file) {
					console.log('[UI] 粘贴的文件:', file.name, file.size, file.type);
					await sendFile(file);
				}
			}
		}

		if (hasFile) {
			console.log('[UI] 已处理粘贴的文件');
		}
	});

	// 添加快捷键提示
	console.log('[UI] Ctrl+V 粘贴文件功能已启用（支持零拷贝）');
}
