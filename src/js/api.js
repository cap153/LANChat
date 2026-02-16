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
    const tauri = getTauri();
    if (tauri) {
        return await tauri.event.listen(eventName, callback);
    } else {
        console.warn(`[JS-API] 当前环境不支持监听事件: ${eventName}`);
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
async function apiSendFile(peerAddr, file) {
    const tauri = getTauri();
    
    if (tauri) {
        // 桌面端 - 使用 Tauri 对话框选择文件
        try {
            console.log("[JS-API] 桌面端发送文件");
            
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
            console.log("[JS-API] 发送到:", peerAddr);
            
            // 调用 Tauri 命令发送文件
            const result = await tauri.core.invoke('send_file', {
                peerAddr,
                filePath
            });
            
            console.log("[JS-API] 文件发送成功:", result);
            return result;
        } catch (e) {
            console.error("[JS-API] 桌面端文件发送失败:", e);
            throw new Error("发送失败: " + e);
        }
    } else {
        // Web 端 - 通过 HTTP 上传
        try {
            // 获取自己的 ID（发送者 ID）
            const myId = await apiGetMyId();
            
            const formData = new FormData();
            formData.append('file', file);
            formData.append('peer_id', myId);  // 传递发送者的 ID
            
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
