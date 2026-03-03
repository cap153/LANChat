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
        return () => {}; // 返回空函数
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
    
    // 根据文件大小选择分块策略
    let baseChunkSize;
    if (fileSize < 100 * 1024 * 1024) {
        // < 100MB：50MB 分块
        baseChunkSize = 50 * 1024 * 1024;
    } else if (fileSize < 500 * 1024 * 1024) {
        // 100-500MB：100MB 分块
        baseChunkSize = 100 * 1024 * 1024;
    } else if (fileSize < 1024 * 1024 * 1024) {
        // 500MB-1GB：200MB 分块
        baseChunkSize = 200 * 1024 * 1024;
    } else if (fileSize < 5 * 1024 * 1024 * 1024) {
        // 1-5GB：300MB 分块
        baseChunkSize = 300 * 1024 * 1024;
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

async function apiSendFile(peerId, peerAddr, file) {
    const tauri = getTauri();
    
    if (tauri) {
        // 桌面端/移动端 - 使用 Tauri 对话框选择文件
        try {
            console.log("[JS-API] Tauri 环境发送文件");
            
            // 使用 Tauri 的文件对话框
            const selected = await tauri.dialog.open({
                multiple: false,
                title: '选择要发送的文件'
            });
            
            if (!selected) {
                throw new Error("未选择文件");
            }
            
            const filePath = Array.isArray(selected) ? selected[0] : selected;
            console.log("[JS-API] 选择的文件:", filePath);
            
            // 检测是否是 content URI (Android)
            const isContentUri = filePath.startsWith('content://');
            console.log("[JS-API] 文件类型:", isContentUri ? "Android content URI" : "普通路径");
            
            let fileData, fileName, fileSize;
            
            if (isContentUri) {
                // Android content URI - 先完整读取文件，再上传
                console.log("[JS-API] 使用流式读取 Android 文件（避免内存溢出）");
                
                // 获取文件元数据以获取真实文件名和大小
                try {
                    const metadata = await tauri.fs.stat(filePath);
                    fileSize = metadata.size;
                    console.log("[JS-API] 文件大小:", fileSize, "字节");
                    
                    // 尝试从元数据获取文件名
                    if (metadata && metadata.name) {
                        fileName = metadata.name;
                    } else {
                        // 如果元数据没有文件名，尝试从 URI 提取
                        const uriParts = filePath.split('/');
                        const lastPart = decodeURIComponent(uriParts[uriParts.length - 1]);
                        
                        // Android document URI 格式: image:1000019150
                        if (lastPart.startsWith('image:')) {
                            fileName = lastPart.split(':')[1] + '.jpg';
                        } else if (lastPart.startsWith('video:')) {
                            fileName = lastPart.split(':')[1] + '.mp4';
                        } else if (lastPart.startsWith('audio:')) {
                            fileName = lastPart.split(':')[1] + '.mp3';
                        } else {
                            fileName = lastPart.includes(':') ? lastPart.split(':')[1] : lastPart;
                        }
                    }
                } catch (statError) {
                    console.warn("[JS-API] 无法获取文件元数据:", statError);
                    fileName = `file_${Date.now()}.dat`;
                    fileSize = 0;
                }
                
                if (!fileName || fileName === '') {
                    fileName = `file_${Date.now()}.dat`;
                }
                
                console.log("[JS-API] 文件名:", fileName, "大小:", fileSize, "字节");
                
                // 立即保存"上传中"消息到数据库，让UI显示
                console.log("[JS-API] 立即保存上传中消息到数据库");
                let messageId = null;
                try {
                    messageId = await tauri.core.invoke('save_file_message', {
                        peerId: peerId,
                        fileName: fileName,
                        fileSize: fileSize,
                        filePath: filePath,
                        status: 'uploading'
                    });
                    console.log("[JS-API] ✓ 上传中消息已保存到数据库，消息ID:", messageId);
                } catch (saveError) {
                    console.error("[JS-API] ✗ 保存上传中消息失败:", saveError);
                }
                
                // 根据设备内存和文件大小计算最优分块大小
                const chunkSize = calculateOptimalChunkSize(fileSize);
                
                // 第一步：获取自己的 ID
                console.log("[JS-API] ========== 第一步：获取用户ID ==========");
                const myId = await apiGetMyId();
                console.log("[JS-API] ✓ 第一步完成：用户ID:", myId);
                
                // 第二步：流式读取并分块上传
                console.log("[JS-API] ========== 第二步：开始流式读取和上传 ==========");
                const uploadUrl = `http://${peerAddr}/api/upload`;
                console.log("[JS-API] 上传地址:", uploadUrl, "文件:", fileName, "大小:", fileSize, "字节");
                
                let offset = 0;
                let chunkCount = 0;
                const startTime = Date.now();
                let lastLogTime = startTime;
                
                try {
                    const file = await tauri.fs.open(filePath, { read: true });
                    
                    while (offset < fileSize) {
                        const size = Math.min(chunkSize, fileSize - offset);
                        const buf = new Uint8Array(size);
                        await file.read(buf, { at: offset });
                        
                        // 构造 FormData 上传这一块
                        const formData = new FormData();
                        // 重要：文本字段必须在文件字段之前
                        formData.append('peer_id', myId);
                        formData.append('file_name', fileName);
                        formData.append('file_size', fileSize.toString());
                        formData.append('chunk_index', chunkCount.toString());
                        formData.append('chunk_total', Math.ceil(fileSize / chunkSize).toString());
                        formData.append('chunk', new Blob([buf], { type: 'application/octet-stream' }), 'chunk');
                        
                        console.log("[JS-API] 上传分块", chunkCount + 1, "大小:", size, "字节");
                        
                        const resp = await fetch(uploadUrl, {
                            method: 'POST',
                            body: formData,
                            mode: 'cors',
                        });
                        
                        if (!resp.ok) {
                            const errorText = await resp.text();
                            console.error("[JS-API] ✗ 上传分块失败，状态码:", resp.status);
                            console.error("[JS-API] ✗ 错误响应体:", errorText);
                            console.error("[JS-API] ✗ 响应头:", resp.headers);
                            throw new Error(`HTTP ${resp.status}: ${errorText}`);
                        }
                        
                        offset += size;
                        chunkCount++;
                        
                        // 每秒打印一次进度
                        const now = Date.now();
                        if (now - lastLogTime > 1000) {
                            const elapsed = (now - startTime) / 1000;
                            const speed = offset / (1024 * 1024) / elapsed;
                            console.log("[JS-API] 已上传:", Math.round(offset / 1024 / 1024), "MB, 速度:", Math.round(speed), "MB/s");
                            lastLogTime = now;
                        }
                    }
                    
                    await file.close();
                    const totalTime = (Date.now() - startTime) / 1000;
                    const avgSpeed = (offset / (1024 * 1024)) / totalTime;
                    console.log("[JS-API] ✓ 第二步完成：文件上传完成，共", chunkCount, "块，总大小:", offset, "字节，耗时:", totalTime.toFixed(2), "秒，平均速度:", avgSpeed.toFixed(2), "MB/s");
                } catch (error) {
                    console.error("[JS-API] ✗ 第二步失败：流式上传失败:", error);
                    throw new Error("文件上传失败: " + error.message);
                }
                
                // 第三步：更新消息状态为已发送
                console.log("[JS-API] ========== 第三步：更新消息状态 ==========");
                try {
                    await tauri.core.invoke('save_file_message', {
                        peerId: peerId,
                        fileName: fileName,
                        fileSize: fileSize,
                        filePath: filePath,
                        status: 'sent'
                    });
                    
                    console.log("[JS-API] ✓ 第三步完成：文件消息状态已更新为已发送");
                } catch (saveError) {
                    console.error("[JS-API] ✗ 第三步失败：更新消息失败:", saveError);
                }
                
                console.log("[JS-API] ========== 文件发送完整流程结束 ==========");
                return {
                    success: true,
                    file_id: '',
                    file_name: fileName,
                    file_size: fileSize,
                };
            } else {
                // 桌面端普通路径 - 直接调用后端命令
                console.log("[JS-API] 桌面端普通路径，调用后端命令");
                const result = await tauri.core.invoke('send_file', {
                    peerId,
                    peerAddr,
                    filePath
                });
                console.log("[JS-API] 文件发送成功:", result);
                return result;
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
            let chunkSize;
            if (navigator.deviceMemory) {
                // 使用设备内存 API（如果可用）
                const deviceMemory = navigator.deviceMemory * 1024 * 1024 * 1024; // 转换为字节
                // 使用可用内存的 80%（大胆使用内存以获得更快的速度）
                const maxChunkMemory = deviceMemory * 0.8;
                
                // 根据文件大小选择基础分块大小
                let baseChunkSize;
                if (fileSize < 100 * 1024 * 1024) {
                    baseChunkSize = 50 * 1024 * 1024;
                } else if (fileSize < 500 * 1024 * 1024) {
                    baseChunkSize = 100 * 1024 * 1024;
                } else if (fileSize < 1024 * 1024 * 1024) {
                    baseChunkSize = 200 * 1024 * 1024;
                } else if (fileSize < 5 * 1024 * 1024 * 1024) {
                    baseChunkSize = 300 * 1024 * 1024;
                } else {
                    baseChunkSize = 500 * 1024 * 1024;
                }
                
                chunkSize = Math.min(baseChunkSize, Math.floor(maxChunkMemory));
                console.log("[JS-API] 设备内存:", Math.round(deviceMemory / (1024 * 1024 * 1024)), "GB");
                console.log("[JS-API] 可用内存预算:", Math.round(maxChunkMemory / (1024 * 1024)), "MB");
            } else {
                // 降级方案：根据文件大小选择分块大小
                if (fileSize < 100 * 1024 * 1024) {
                    chunkSize = 50 * 1024 * 1024;
                } else if (fileSize < 500 * 1024 * 1024) {
                    chunkSize = 100 * 1024 * 1024;
                } else if (fileSize < 1024 * 1024 * 1024) {
                    chunkSize = 200 * 1024 * 1024;
                } else if (fileSize < 5 * 1024 * 1024 * 1024) {
                    chunkSize = 300 * 1024 * 1024;
                } else {
                    chunkSize = 500 * 1024 * 1024;
                }
            }
            
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
                
                // 每秒打印一次进度
                const now = Date.now();
                if (now - lastLogTime > 1000) {
                    const elapsed = (now - startTime) / 1000;
                    const speed = offset / (1024 * 1024) / elapsed;
                    console.log("[JS-API] 已上传:", Math.round(offset / 1024 / 1024), "MB, 速度:", Math.round(speed), "MB/s");
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