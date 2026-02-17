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
                // Android content URI - 使用 Tauri fs 插件读取
                console.log("[JS-API] 使用 Tauri fs 插件读取 Android 文件");
                
                // 使用 Tauri fs 插件读取文件
                try {
                    fileData = await tauri.fs.readFile(filePath);
                    fileSize = fileData.length;
                    console.log("[JS-API] 文件读取成功:", fileSize, "字节");
                } catch (fsError) {
                    console.error("[JS-API] Tauri fs 读取失败:", fsError);
                    throw new Error("无法读取文件: " + fsError.message);
                }
                
                // 获取文件元数据以获取真实文件名
                try {
                    const metadata = await tauri.fs.stat(filePath);
                    console.log("[JS-API] 文件元数据:", metadata);
                    
                    // 尝试从元数据获取文件名
                    if (metadata && metadata.name) {
                        fileName = metadata.name;
                    } else {
                        // 如果元数据没有文件名，尝试从 URI 提取
                        const uriParts = filePath.split('/');
                        const lastPart = decodeURIComponent(uriParts[uriParts.length - 1]);
                        
                        // Android document URI 格式: image:1000019150
                        // 我们需要猜测扩展名
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
                    // 降级方案：使用时间戳作为文件名
                    fileName = `file_${Date.now()}.dat`;
                }
                
                if (!fileName || fileName === '') {
                    fileName = `file_${Date.now()}.dat`;
                }
                
                console.log("[JS-API] 最终文件名:", fileName);
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
            
            // Android: 将文件数据发送到对方服务器
            console.log("[JS-API] 准备上传 Android 文件到:", peerAddr);
            
            // 获取自己的 ID
            const myId = await apiGetMyId();
            
            // 构造 FormData
            const formData = new FormData();
            formData.append('peer_id', myId);
            formData.append('file', new Blob([fileData]), fileName);
            
            // 上传到对方的服务器
            const uploadUrl = `http://${peerAddr}/api/upload`;
            console.log("[JS-API] 上传到:", uploadUrl, "文件:", fileName, fileSize, "字节");
            
            const resp = await fetch(uploadUrl, {
                method: 'POST',
                body: formData,
                mode: 'cors',
            });
            
            console.log("[JS-API] 响应状态:", resp.status);
            
            if (!resp.ok) {
                const errorText = await resp.text();
                console.error("[JS-API] 上传失败:", errorText);
                throw new Error(`HTTP ${resp.status}: ${errorText}`);
            }
            
            const result = await resp.json();
            console.log("[JS-API] Android 文件上传成功:", result);
            
            // Android 端需要手动保存消息到数据库
            try {
                console.log("[JS-API] 保存文件发送记录到数据库");
                
                // 调用后端保存消息
                await tauri.core.invoke('save_file_message', {
                    peerId: peerId,
                    fileName: fileName,
                    fileSize: fileSize,
                    filePath: filePath,
                    status: 'sent'
                });
                
                console.log("[JS-API] 文件消息已保存到数据库");
            } catch (saveError) {
                console.error("[JS-API] 保存消息失败:", saveError);
                // 不影响文件发送结果
            }
            
            return {
                success: true,
                file_id: result.file_id || '',
                file_name: fileName,
                file_size: fileSize,
            };
            
        } catch (e) {
            console.error("[JS-API] 文件发送失败:", e);
            throw new Error("发送失败: " + e.message);
        }
    } else {
        // Web 端 - 通过 HTTP 上传
        try {
            // 获取自己的 ID（发送者 ID）
            const myId = await apiGetMyId();
            
            const formData = new FormData();
            formData.append('peer_id', myId);  // 传递发送者的 ID（必须在 file 之前）
            formData.append('file', file);
            
            // 上传到对方的服务器
            const uploadUrl = `http://${peerAddr}/api/upload`;
            console.log("[JS-API] 上传文件到:", uploadUrl);
            console.log("[JS-API] 文件信息:", file.name, file.size, file.type);
            console.log("[JS-API] sender_id (我的ID):", myId);
            
            const resp = await fetch(uploadUrl, {
                method: 'POST',
                body: formData,
                mode: 'cors',
            });
            
            console.log("[JS-API] 响应状态:", resp.status, resp.statusText);
            
            if (!resp.ok) {
                const errorText = await resp.text();
                console.error("[JS-API] 错误响应:", errorText);
                throw new Error(`HTTP ${resp.status}: ${errorText}`);
            }
            
            const data = await resp.json();
            console.log("[JS-API] 文件上传成功:", data);
            return data;
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