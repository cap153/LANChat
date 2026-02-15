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
