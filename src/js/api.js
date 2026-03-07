// 防御性获取 Tauri 接口
const getTauri = () => window.__TAURI__;

async function apiGetMyName() {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端环境
		try {
			console.log("[JS-API] 正在通过 Tauri 调用 get_my_name");
			return await tauri.core.invoke('get_my_name');
		} catch (e) {
			console.error("[JS-API] Tauri 调用失败:", e);
			return "Tauri错误";
		}
	} else {
		// Web 端环境
		try {
			console.log("[JS-API] 正在通过 HTTP 调用 get_my_name");
			const resp = await fetch('/api/get_my_name');
			const data = await resp.json();
			return data.name;
		} catch (e) {
			console.error("[JS-API] Web 请求失败:", e);
			return "Web未连接";
		}
	}
}

async function apiGetMyId() {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端环境
		try {
			console.log("[JS-API] 正在通过 Tauri 调用 get_my_id");
			return await tauri.core.invoke('get_my_id');
		} catch (e) {
			console.error("[JS-API] Tauri 调用失败:", e);
			throw new Error("获取 ID 失败: " + e);
		}
	} else {
		// Web 端环境
		try {
			console.log("[JS-API] 正在通过 HTTP 调用 get_my_id");
			const resp = await fetch('/api/get_my_id');
			const data = await resp.json();
			return data.id;
		} catch (e) {
			console.error("[JS-API] Web 请求失败:", e);
			throw new Error("获取 ID 失败: " + e);
		}
	}
}

// 获取设置
async function apiGetSettings() {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端
		try {
			console.log("[JS-API] 通过 Tauri 获取设置");
			return await tauri.core.invoke('get_settings');
		} catch (e) {
			console.error("[JS-API] 获取设置失败:", e);
			throw new Error("获取设置失败: " + e);
		}
	} else {
		// Web 端
		try {
			console.log("[JS-API] 通过 HTTP 获取设置");
			const resp = await fetch('/api/get_settings');
			const data = await resp.json();
			return data;
		} catch (e) {
			console.error("[JS-API] 获取设置失败:", e);
			throw new Error("获取设置失败: " + e);
		}
	}
}

// 更新设置
async function apiUpdateSettings(downloadPath) {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端
		try {
			console.log("[JS-API] 通过 Tauri 更新设置");
			return await tauri.core.invoke('update_settings', {
				downloadPath
			});
		} catch (e) {
			console.error("[JS-API] 更新设置失败:", e);
			throw new Error("更新设置失败: " + e);
		}
	} else {
		// Web 端
		try {
			console.log("[JS-API] 通过 HTTP 更新设置");
			const resp = await fetch('/api/update_settings', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					download_path: downloadPath
				})
			});
			const data = await resp.json();
			if (data.error) {
				throw new Error(data.error);
			}
			return data;
		} catch (e) {
			console.error("[JS-API] 更新设置失败:", e);
			throw new Error("更新设置失败: " + e);
		}
	}
}

// 获取默认下载路径
async function apiGetDefaultDownloadPath() {
	const tauri = getTauri();

	if (tauri) {
		try {
			return await tauri.core.invoke('get_default_download_path');
		} catch (e) {
			console.error("[JS-API] 获取默认路径失败:", e);
			return '/storage/emulated/0/Download/LANChat';
		}
	} else {
		return '/tmp/lanchat';
	}
}

async function apiUpdateMyName(newName) {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端环境
		try {
			console.log("[JS-API] 正在通过 Tauri 调用 update_my_name");
			return await tauri.core.invoke('update_my_name', { newName });
		} catch (e) {
			console.error("[JS-API] Tauri 调用失败:", e);
			throw new Error("更新失败: " + e);
		}
	} else {
		// Web 端环境
		try {
			console.log("[JS-API] 正在通过 HTTP 调用 update_my_name");
			const resp = await fetch('/api/update_my_name', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: newName })
			});
			const data = await resp.json();
			if (data.error) {
				throw new Error(data.error);
			}
			return data.name;
		} catch (e) {
			console.error("[JS-API] Web 请求失败:", e);
			throw new Error("更新失败: " + e);
		}
	}
}

// 导出监听函数，同样要做环境判断
async function apiListen(eventName, callback) {
	console.log("[JS-API] 尝试监听事件:", eventName);
	const tauri = getTauri();
	if (tauri) {
		console.log("[JS-API] ✓ Tauri 环境，注册事件监听器:", eventName);
		const unlisten = await tauri.event.listen(eventName, callback);
		console.log("[JS-API] ✓ 事件监听器注册成功:", eventName);
		return unlisten;
	} else {
		console.warn(`[JS-API] ✗ 当前环境不支持监听事件: ${eventName}`);
		return () => { }; // 返回空函数
	}
}

// 获取在线用户列表（仅 Web 端使用）
async function apiGetPeers() {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端通过 Tauri 命令获取
		try {
			return await tauri.core.invoke('get_peers');
		} catch (e) {
			console.error("[JS-API] 桌面端获取用户列表失败:", e);
			return [];
		}
	} else {
		// Web 端通过 HTTP 轮询
		try {
			const resp = await fetch('/api/get_peers');

			if (!resp.ok) {
				console.error("[JS-API] HTTP 错误:", resp.status, resp.statusText);
				return [];
			}

			const text = await resp.text();
			console.log("[JS-API] 收到响应:", text);

			if (!text) {
				console.warn("[JS-API] 响应为空");
				return [];
			}

			const peers = JSON.parse(text);
			return peers;
		} catch (e) {
			console.error("[JS-API] 获取用户列表失败:", e);
			return [];
		}
	}
}


// 发送文本消息
async function apiSendMessage(peerId, peerAddr, content) {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端
		try {
			console.log("[JS-API] 通过 Tauri 发送消息");
			return await tauri.core.invoke('send_message', {
				peerId,
				peerAddr,
				content
			});
		} catch (e) {
			console.error("[JS-API] 发送消息失败:", e);
			throw new Error("发送失败: " + e);
		}
	} else {
		// Web 端
		try {
			console.log("[JS-API] 通过 HTTP 发送消息");
			const resp = await fetch('/api/send_message', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					peer_id: peerId,     // 添加接收者ID
					peer_addr: peerAddr,
					content
				})
			});
			const data = await resp.json();
			if (data.error) {
				throw new Error(data.error);
			}
			return data;
		} catch (e) {
			console.error("[JS-API] 发送消息失败:", e);
			throw new Error("发送失败: " + e);
		}
	}
}

// 获取聊天历史
async function apiGetChatHistory(peerId) {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端
		try {
			return await tauri.core.invoke('get_chat_history', { peerId });
		} catch (e) {
			console.error("[JS-API] 获取历史消息失败:", e);
			return [];
		}
	} else {
		// Web 端
		try {
			const resp = await fetch(`/api/chat_history/${peerId}`);

			if (!resp.ok) {
				console.error("[JS-API] HTTP 错误:", resp.status, resp.statusText);
				return [];
			}

			const text = await resp.text();
			console.log("[JS-API] 收到历史消息响应:", text.substring(0, 200));

			if (!text) {
				console.warn("[JS-API] 响应为空");
				return [];
			}

			const data = JSON.parse(text);
			return data.messages || [];
		} catch (e) {
			console.error("[JS-API] 获取历史消息失败:", e);
			return [];
		}
	}
}


// 发送文件
// 获取设备可用内存（估算）
function getAvailableMemory() {
	if (navigator.deviceMemory) {
		// 使用 Device Memory API（如果可用）
		return navigator.deviceMemory * 1024 * 1024 * 1024; // 转换为字节
	}
	// 默认估算：假设设备有 2GB 内存
	return 2 * 1024 * 1024 * 1024;
}

// 根据设备内存和文件大小计算最优分块大小
function calculateOptimalChunkSize(fileSize) {
	const availableMemory = getAvailableMemory();
	// 使用可用内存的 80%（大胆使用内存以获得更快的速度）
	const maxChunkMemory = availableMemory * 0.8;

	// 根据文件大小选择分块策略，基础大小调大
	let baseChunkSize;
	if (fileSize < 100 * 1024 * 1024) {
		// < 100MB：100MB 分块
		baseChunkSize = 100 * 1024 * 1024;
	} else if (fileSize < 500 * 1024 * 1024) {
		// 100-500MB：200MB 分块
		baseChunkSize = 200 * 1024 * 1024;
	} else if (fileSize < 1024 * 1024 * 1024) {
		// 500MB-1GB：300MB 分块
		baseChunkSize = 300 * 1024 * 1024;
	} else if (fileSize < 5 * 1024 * 1024 * 1024) {
		// 1-5GB：400MB 分块
		baseChunkSize = 400 * 1024 * 1024;
	} else {
		// > 5GB：500MB 分块
		baseChunkSize = 500 * 1024 * 1024;
	}

	// 根据可用内存调整分块大小（不超过可用内存的 80%）
	const chunkSize = Math.min(baseChunkSize, Math.floor(maxChunkMemory));

	console.log("[JS-API] 设备内存:", Math.round(availableMemory / (1024 * 1024 * 1024)), "GB");
	console.log("[JS-API] 可用内存预算:", Math.round(maxChunkMemory / (1024 * 1024)), "MB");
	console.log("[JS-API] 计算的分块大小:", Math.round(chunkSize / (1024 * 1024)), "MB");

	return chunkSize;
}

async function apiSendFile(peerId, peerAddr, file, filePath) {
	const tauri = getTauri();

	if (tauri) {
		// 桌面端/移动端 - 使用 Tauri 对话框选择文件或使用提供的路径
		try {
			console.log("[JS-API] Tauri 环境发送文件");

			let selectedPath = filePath;

			// 如果没有提供文件路径，使用对话框选择
			if (!selectedPath) {
				const selected = await tauri.dialog.open({
					multiple: false,
					title: '选择要发送的文件'
				});

				if (!selected) {
					throw new Error("未选择文件");
				}

				selectedPath = Array.isArray(selected) ? selected[0] : selected;
			}

			console.log("[JS-API] 文件路径:", selectedPath);

			// 监听后端传来的进度事件
			let unlistenProgress;
			if (tauri.event) {
				unlistenProgress = await tauri.event.listen('upload_progress', (event) => {
					const speed = event.payload.speed_mb_s;
					// 更新 DOM 显示速度
					const statusDivs = document.querySelectorAll('.file-uploading');
					statusDivs.forEach(div => {
						div.textContent = Math.round(speed) + ' MB/s';
					});
				});
			}

			try {
				// 直接调用后端统一命令，后端会自动处理 content URI 和普通路径
				const result = await tauri.core.invoke('send_file', {
					peerId,
					peerAddr,
					filePath: selectedPath
				});
				console.log("[JS-API] 文件发送成功:", result);

				// 修改 DOM 取消上传中状态
				const statusDivs = document.querySelectorAll('.file-uploading');
				statusDivs.forEach(div => {
					div.textContent = '已发送';
					div.classList.remove('file-uploading');
				});

				return result;
			} finally {
				// 取消事件监听
				if (unlistenProgress) {
					unlistenProgress();
				}
			}

		} catch (e) {
			console.error("[JS-API] 文件发送失败:", e);
			throw new Error("发送失败: " + e.message);
		}
	} else {
		// Web 端 - 通过 HTTP 上传（使用分块协议）
		try {
			// 获取自己的 ID（发送者 ID）
			const myId = await apiGetMyId();

			const fileName = file.name;
			const fileSize = file.size;

			console.log("[JS-API] Web 端分块上传");
			console.log("[JS-API] 文件信息:", fileName, fileSize, "字节");
			console.log("[JS-API] sender_id (我的ID):", myId);

			// 根据设备内存动态计算分块大小
			const chunkSize = calculateOptimalChunkSize(fileSize);
			const totalChunks = Math.ceil(fileSize / chunkSize);

			console.log("[JS-API] 计算的分块大小:", Math.round(chunkSize / (1024 * 1024)), "MB, 总分块数:", totalChunks);

			const uploadUrl = `http://${peerAddr}/api/upload`;
			console.log("[JS-API] 上传地址:", uploadUrl);

			let offset = 0;
			let chunkIndex = 0;
			const startTime = Date.now();
			let lastLogTime = startTime;

			while (offset < fileSize) {
				const size = Math.min(chunkSize, fileSize - offset);
				const chunk = file.slice(offset, offset + size);

				// 构造 FormData 上传这一块
				const formData = new FormData();
				formData.append('peer_id', myId);
				formData.append('file_name', fileName);
				formData.append('file_size', fileSize.toString());
				formData.append('chunk_index', chunkIndex.toString());
				formData.append('chunk_total', totalChunks.toString());
				formData.append('chunk', chunk, 'chunk');

				console.log("[JS-API] 上传分块", chunkIndex + 1, "大小:", size, "字节");

				const resp = await fetch(uploadUrl, {
					method: 'POST',
					body: formData,
					mode: 'cors',
				});

				if (!resp.ok) {
					const errorText = await resp.text();
					console.error("[JS-API] ✗ 上传分块失败，状态码:", resp.status);
					console.error("[JS-API] ✗ 错误响应:", errorText);
					throw new Error(`HTTP ${resp.status}: ${errorText}`);
				}

				offset += size;
				chunkIndex++;

				// 每秒打印一次进度并更新 UI
				const now = Date.now();
				if (now - lastLogTime > 1000) {
					const elapsed = (now - startTime) / 1000;
					const speed = offset / (1024 * 1024) / elapsed;
					console.log("[JS-API] 已上传:", Math.round(offset / 1024 / 1024), "MB, 速度:", Math.round(speed), "MB/s");

					// 更新 UI 中的速度显示
					const statusDivs = document.querySelectorAll('.file-uploading');
					statusDivs.forEach(div => {
						div.textContent = Math.round(speed) + ' MB/s';
					});

					lastLogTime = now;
				}
			}

			const totalTime = (Date.now() - startTime) / 1000;
			const avgSpeed = (fileSize / (1024 * 1024)) / totalTime;
			console.log("[JS-API] ✓ 文件上传完成，共", chunkIndex, "块，耗时:", totalTime.toFixed(2), "秒，平均速度:", avgSpeed.toFixed(2), "MB/s");

			return {
				success: true,
				file_name: fileName,
				file_size: fileSize,
			};
		} catch (e) {
			console.error("[JS-API] 文件上传失败:", e);
			throw new Error("上传失败: " + e.message);
		}
	}
}
// 主题相关 API
async function apiGetThemeList() {
	const tauri = window.__TAURI__;
	if (tauri) {
		// 桌面端
		return await tauri.core.invoke('get_theme_list');
	} else {
		// Web 端
		const response = await fetch('/api/get_theme_list');
		if (!response.ok) {
			throw new Error('获取主题列表失败');
		}
		return await response.json();
	}
}

async function apiGetThemeCss(themeName) {
	const tauri = window.__TAURI__;
	if (tauri) {
		// 桌面端
		return await tauri.core.invoke('get_theme_css', { themeName });
	} else {
		// Web 端
		const response = await fetch(`/api/get_theme_css/${themeName}`);
		if (!response.ok) {
			throw new Error('获取主题CSS失败');
		}
		return await response.text();
	}
}

async function apiSaveCurrentTheme(themeName) {
	const tauri = window.__TAURI__;
	if (tauri) {
		// 桌面端
		return await tauri.core.invoke('save_current_theme', { themeName });
	} else {
		// Web 端
		const response = await fetch('/api/save_current_theme', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ theme_name: themeName })
		});
		if (!response.ok) {
			throw new Error('保存主题失败');
		}
		return await response.json();
	}
}

async function apiGetCurrentTheme() {
	const tauri = window.__TAURI__;
	if (tauri) {
		// 桌面端
		return await tauri.core.invoke('get_current_theme');
	} else {
		// Web 端
		const response = await fetch('/api/get_current_theme');
		if (!response.ok) {
			throw new Error('获取当前主题失败');
		}
		const result = await response.json();
		return result.theme;
	}
}

// Android 分享相关 API
async function apiGetAndroidSharedFiles() {
	console.log("[JS-API] apiGetAndroidSharedFiles 被调用");
	
	const tauri = getTauri();
	
	if (tauri) {
		// 使用 Tauri 命令获取分享文件
		try {
			console.log("[JS-API] 通过 Tauri 命令获取分享文件");
			const files = await tauri.core.invoke('get_android_shared_files');
			console.log("[JS-API] Tauri 返回的文件:", files);
			return files;
		} catch (e) {
			console.error("[JS-API] Tauri 命令失败:", e);
		}
	}
	
	// 降级方案：使用 window.Android
	console.log("[JS-API] window.Android 存在:", !!window.Android);
	
	if (window.Android && window.Android.getPendingSharedFiles) {
		try {
			console.log("[JS-API] 调用 window.Android.getPendingSharedFiles()");
			const jsonStr = window.Android.getPendingSharedFiles();
			console.log("[JS-API] 返回的 JSON 字符串:", jsonStr);
			const files = JSON.parse(jsonStr);
			console.log("[JS-API] 解析后的文件:", files);
			return files;
		} catch (e) {
			console.error("[JS-API] 获取 Android 分享文件失败:", e);
			return [];
		}
	}
	console.log("[JS-API] 所有方法都不可用，返回空数组");
	return [];
}

async function apiClearAndroidSharedFiles() {
	const tauri = getTauri();
	
	if (tauri) {
		try {
			await tauri.core.invoke('clear_android_shared_files');
			return;
		} catch (e) {
			console.error("[JS-API] 清除分享文件失败:", e);
		}
	}
	
	if (window.Android && window.Android.clearPendingSharedFiles) {
		window.Android.clearPendingSharedFiles();
	}
}

async function apiSendFileFromAndroidUri(peerId, peerAddr, fileInfo) {
	const tauri = getTauri();
	
	if (!tauri) {
		throw new Error("Tauri 不可用");
	}

	try {
		console.log("[JS-API] 从 Android URI 发送文件:", fileInfo);
		console.log("[JS-API] peerId:", peerId, "peerAddr:", peerAddr);

		// 确保文件名不为空
		let fileName = fileInfo.fileName;
		if (!fileName || fileName.trim() === '') {
			// 根据 MIME 类型生成默认文件名
			const timestamp = Date.now();
			const ext = fileInfo.mimeType ? fileInfo.mimeType.split('/')[1] || 'dat' : 'dat';
			fileName = `shared_file_${timestamp}.${ext}`;
			console.log("[JS-API] 文件名为空，生成默认文件名:", fileName);
		}

		// 检查是否有文件描述符
		if (fileInfo.fd && fileInfo.fd >= 0) {
			console.log("[JS-API] 使用文件描述符发送: fd=" + fileInfo.fd);
			
			// 使用 FD 发送文件
			const params = {
				peerId: peerId,
				peerAddr: peerAddr,
				fileName: fileName,
				fileSize: fileInfo.fileSize,
				fd: fileInfo.fd,
				originalUri: fileInfo.uri || null  // 传递原始 URI
			};
			
			console.log("[JS-API] 调用 send_file_from_fd，参数:", JSON.stringify(params));
			
			const result = await tauri.core.invoke('send_file_from_fd', params);
			console.log("[JS-API] 文件发送成功:", result);
			return result;
		} else {
			// 没有 FD，无法发送
			console.error("[JS-API] 没有有效的文件描述符: fd=" + fileInfo.fd);
			throw new Error("无法获取文件描述符，无法发送文件");
		}
	} catch (e) {
		console.error("[JS-API] 从 Android URI 发送文件失败:", e);
		throw e;
	}
}

// 分享文件到其他应用（仅 Android）
async function apiShareFileToOtherApp(filePath) {
	const tauri = getTauri();
	
	if (!tauri) {
		throw new Error("仅支持 Android 端");
	}
	
	try {
		console.log("[JS-API] 分享文件到其他应用:", filePath);
		await tauri.core.invoke('share_file_to_other_app', { filePath });
		console.log("[JS-API] 分享成功");
	} catch (e) {
		console.error("[JS-API] 分享文件失败:", e);
		throw e;
	}
}

