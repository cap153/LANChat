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
