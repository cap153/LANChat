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
function addUserToList(name, addr) {
    const list = document.getElementById('user-list');
    if (!list) return;
    
    // 检查是否已存在
    const existingItems = list.querySelectorAll('li');
    for (let item of existingItems) {
        if (item.dataset.addr === addr) {
            return; // 已存在，不重复添加
        }
    }
    
    const li = document.createElement('li');
    li.dataset.addr = addr;
    li.innerHTML = `
        <span class="user-name">${name}</span>
        <span class="user-addr">${addr}</span>
    `;
    list.appendChild(li);
    
    console.log('[UI] 添加用户到列表:', name, addr);
}
