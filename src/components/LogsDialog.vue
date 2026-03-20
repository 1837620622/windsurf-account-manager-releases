<template>
  <el-dialog
    v-model="uiStore.showLogsDialog"
    title="操作日志"
    width="800px"
  >
    <div class="logs-container">
      <div class="logs-header">
        <el-button size="small" @click="loadLogs" :icon="Refresh">
          刷新
        </el-button>
        <el-button size="small" @click="clearLogs" :icon="Delete">
          清空日志
        </el-button>
      </div>
      
      <el-table :data="logs" style="width: 100%" max-height="400">
        <el-table-column prop="timestamp" label="时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.timestamp) }}
          </template>
        </el-table-column>
        
        <el-table-column prop="operation_type" label="操作类型" width="120">
          <template #default="{ row }">
            <el-tag :type="getOperationTypeTag(row.operation_type)">
              {{ formatOperationType(row.operation_type) }}
            </el-tag>
          </template>
        </el-table-column>
        
        <el-table-column prop="account_email" label="账号" width="180" />
        
        <el-table-column prop="message" label="消息" />
        
        <el-table-column prop="status" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'">
              {{ row.status === 'success' ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, Delete } from '@element-plus/icons-vue';
import { useSettingsStore, useUIStore } from '@/store';
import dayjs from 'dayjs';

const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const logs = computed(() => {
  // 按时间倒序排列，最新的在前面
  return [...settingsStore.logs].sort((a, b) => {
    return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
  });
});

onMounted(() => {
  loadLogs();
});

async function loadLogs() {
  try {
    await settingsStore.loadLogs(100);
  } catch (error) {
    ElMessage.error(`加载日志失败: ${error}`);
  }
}

async function clearLogs() {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有日志吗？',
      '清空日志',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    
    await settingsStore.clearLogs();
    ElMessage.success('日志已清空');
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`清空日志失败: ${error}`);
    }
  }
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
}

function formatOperationType(type: string) {
  const typeMap: Record<string, string> = {
    login: '登录',
    refresh_token: '刷新Token',
    reset_credits: '重置积分',
    update_seats: '更新座位',
    get_billing: '查询账单',
    update_plan: '更新计划',
    add_account: '添加账号',
    delete_account: '删除账号',
    edit_account: '编辑账号',
    batch_operation: '批量操作',
  };
  return typeMap[type] || type;
}

function getOperationTypeTag(type: string) {
  const tagMap: Record<string, string> = {
    login: 'primary',
    refresh_token: 'info',
    reset_credits: 'success',
    update_seats: 'warning',
    get_billing: 'info',
    update_plan: 'warning',
    add_account: 'success',
    delete_account: 'danger',
    edit_account: 'warning',
    batch_operation: 'primary',
  };
  return tagMap[type] || 'info';
}
</script>

<style scoped>
.logs-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.logs-header {
  display: flex;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
}
</style>
