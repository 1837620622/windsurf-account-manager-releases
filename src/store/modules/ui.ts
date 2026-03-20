import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useUIStore = defineStore('ui', () => {
  const sidebarCollapsed = ref(true);  // 默认收缩
  // 从localStorage恢复主题，如果没有则默认为light
  const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
  const theme = ref<'light' | 'dark'>(savedTheme || 'light');
  
  // 初始化时立即应用主题
  if (savedTheme === 'dark') {
    document.documentElement.classList.add('dark');
    document.documentElement.classList.add('el-theme-dark');
  }
  const showAddAccountDialog = ref(false);
  const showEditAccountDialog = ref(false);
  const showSettingsDialog = ref(false);
  const showLogsDialog = ref(false);
  const showBatchOperationDialog = ref(false);
  const showStatsDialog = ref(false);
  const showAccountInfoDialog = ref(false);
  const showBillingDialog = ref(false);
  const currentEditingAccountId = ref<string | null>(null);
  const currentViewingAccountId = ref<string | null>(null);
  
  // 通知相关
  const notifications = ref<Array<{
    id: string;
    type: 'success' | 'warning' | 'error' | 'info';
    title: string;
    message?: string;
    duration?: number;
  }>>([]);

  // Actions
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  async function setTheme(newTheme: 'light' | 'dark') {
    theme.value = newTheme;
    
    // 保存到localStorage
    localStorage.setItem('theme', newTheme);
    
    // 设置HTML根元素的class
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark');
      // Element Plus 深色模式
      document.documentElement.classList.add('el-theme-dark');
      document.documentElement.style.setProperty('--el-bg-color', '#1d1e1f');
      document.documentElement.style.setProperty('--el-bg-color-page', '#0c0d0e');
      document.documentElement.style.setProperty('--el-color-primary', '#409eff');
      document.documentElement.style.setProperty('--el-color-white', '#1d1e1f');
      document.documentElement.style.setProperty('--el-text-color-primary', '#e5eaf3');
      document.documentElement.style.setProperty('--el-text-color-regular', '#cfd3dc');
      document.documentElement.style.setProperty('--el-border-color', '#4c4d4f');
    } else {
      document.documentElement.classList.remove('dark');
      document.documentElement.classList.remove('el-theme-dark');
      // 恢复默认颜色
      document.documentElement.style.removeProperty('--el-bg-color');
      document.documentElement.style.removeProperty('--el-bg-color-page');
      document.documentElement.style.removeProperty('--el-color-primary');
      document.documentElement.style.removeProperty('--el-color-white');
      document.documentElement.style.removeProperty('--el-text-color-primary');
      document.documentElement.style.removeProperty('--el-text-color-regular');
      document.documentElement.style.removeProperty('--el-border-color');
    }
    
    // 保存主题设置到后端
    try {
      const { settingsApi } = await import('@/api');
      // 获取当前设置并更新主题
      const currentSettings = await settingsApi.getSettings();
      await settingsApi.updateSettings({ ...currentSettings, theme: newTheme });
    } catch (error) {
      console.error('Failed to save theme setting:', error);
    }
  }

  function showNotification(notification: Omit<typeof notifications.value[0], 'id'>) {
    const id = Date.now().toString();
    notifications.value.push({ ...notification, id });
    
    if (notification.duration !== 0) {
      setTimeout(() => {
        removeNotification(id);
      }, notification.duration || 3000);
    }
  }

  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id);
    if (index > -1) {
      notifications.value.splice(index, 1);
    }
  }

  function openAddAccountDialog() {
    showAddAccountDialog.value = true;
  }

  function closeAddAccountDialog() {
    showAddAccountDialog.value = false;
  }

  function openEditAccountDialog(accountId: string) {
    currentEditingAccountId.value = accountId;
    showEditAccountDialog.value = true;
  }

  function closeEditAccountDialog() {
    showEditAccountDialog.value = false;
    currentEditingAccountId.value = null;
  }

  function openSettingsDialog() {
    showSettingsDialog.value = true;
  }

  function closeSettingsDialog() {
    showSettingsDialog.value = false;
  }

  function openLogsDialog() {
    showLogsDialog.value = true;
  }

  function closeLogsDialog() {
    showLogsDialog.value = false;
  }

  function openBatchOperationDialog() {
    showBatchOperationDialog.value = true;
  }

  function closeBatchOperationDialog() {
    showBatchOperationDialog.value = false;
  }

  function openStatsDialog() {
    showStatsDialog.value = true;
  }

  function closeStatsDialog() {
    showStatsDialog.value = false;
  }

  function openAccountInfoDialog(accountId: string) {
    currentViewingAccountId.value = accountId;
    showAccountInfoDialog.value = true;
  }

  function closeAccountInfoDialog() {
    showAccountInfoDialog.value = false;
    currentViewingAccountId.value = null;
  }

  function openBillingDialog(accountId: string) {
    currentViewingAccountId.value = accountId;
    showBillingDialog.value = true;
  }

  function closeBillingDialog() {
    showBillingDialog.value = false;
    currentViewingAccountId.value = null;
  }

  return {
    // State
    sidebarCollapsed,
    theme,
    showAddAccountDialog,
    showEditAccountDialog,
    showSettingsDialog,
    showLogsDialog,
    showBatchOperationDialog,
    showStatsDialog,
    showAccountInfoDialog,
    showBillingDialog,
    currentEditingAccountId,
    currentViewingAccountId,
    notifications,

    // Actions
    toggleSidebar,
    setTheme,
    openAddAccountDialog,
    closeAddAccountDialog,
    openEditAccountDialog,
    closeEditAccountDialog,
    openSettingsDialog,
    closeSettingsDialog,
    openLogsDialog,
    closeLogsDialog,
    openBatchOperationDialog,
    closeBatchOperationDialog,
    openStatsDialog,
    closeStatsDialog,
    openAccountInfoDialog,
    closeAccountInfoDialog,
    openBillingDialog,
    closeBillingDialog,
    showNotification,
    removeNotification,
  };
});
