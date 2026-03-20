<template>
  <el-dialog
    v-model="uiStore.showStatsDialog"
    title="统计信息"
    width="700px"
  >
    <div v-if="loading" class="loading-container">
      <el-icon class="is-loading" size="32"><Loading /></el-icon>
    </div>
    
    <div v-else-if="stats">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-statistic title="账号总数" :value="stats.total_accounts" />
        </el-col>
        <el-col :span="8">
          <el-statistic title="活跃账号" :value="stats.active_accounts" />
        </el-col>
        <el-col :span="8">
          <el-statistic title="分组数量" :value="stats.groups" />
        </el-col>
      </el-row>
      
      <el-divider />
      
      <el-row :gutter="20">
        <el-col :span="12">
          <el-statistic 
            title="操作成功率" 
            :value="stats.success_rate" 
            suffix="%" 
            :precision="1"
          />
          <div class="stat-detail">
            成功: {{ stats.successful_operations }} / 失败: {{ stats.failed_operations }}
          </div>
        </el-col>
        <el-col :span="12">
          <el-statistic 
            title="积分重置成功率" 
            :value="stats.reset_success_rate" 
            suffix="%" 
            :precision="1"
          />
          <div class="stat-detail">
            成功: {{ stats.successful_resets }} / 失败: {{ stats.failed_resets }}
          </div>
        </el-col>
      </el-row>
      
      <el-divider />
      
      <el-descriptions :column="1" border>
        <el-descriptions-item label="总操作次数">
          {{ stats.total_operations }}
        </el-descriptions-item>
        
        <el-descriptions-item label="总重置次数">
          {{ stats.total_resets }}
        </el-descriptions-item>
        
        <el-descriptions-item label="最后操作时间">
          {{ stats.last_operation ? formatDate(stats.last_operation) : '暂无' }}
        </el-descriptions-item>
        
        <el-descriptions-item label="自动刷新Token">
          <el-tag :type="stats.settings?.auto_refresh_token ? 'success' : 'info'">
            {{ stats.settings?.auto_refresh_token ? '开启' : '关闭' }}
          </el-tag>
        </el-descriptions-item>
        
        <el-descriptions-item label="重试次数">
          {{ stats.settings?.retry_times || 2 }}
        </el-descriptions-item>
        
        <el-descriptions-item label="并发限制">
          {{ stats.settings?.concurrent_limit || 5 }}
        </el-descriptions-item>
      </el-descriptions>
    </div>
    
    <template #footer>
      <el-button @click="refresh" :icon="Refresh">刷新</el-button>
      <el-button @click="uiStore.closeStatsDialog">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { Loading, Refresh } from '@element-plus/icons-vue';
import { useUIStore } from '@/store';
import { settingsApi } from '@/api';
import dayjs from 'dayjs';

const uiStore = useUIStore();

const loading = ref(false);
const stats = ref<any>(null);

watch(() => uiStore.showStatsDialog, (show) => {
  if (show) {
    loadStats();
  }
});

onMounted(() => {
  if (uiStore.showStatsDialog) {
    loadStats();
  }
});

async function loadStats() {
  loading.value = true;
  try {
    stats.value = await settingsApi.getStats();
  } catch (error) {
    ElMessage.error(`加载统计信息失败: ${error}`);
  } finally {
    loading.value = false;
  }
}

function refresh() {
  loadStats();
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
}
</script>

<style scoped>
.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px;
}

.stat-detail {
  margin-top: 8px;
  font-size: 12px;
  color: #909399;
  text-align: center;
}

.el-statistic {
  text-align: center;
}

.el-row {
  margin-bottom: 20px;
}
</style>
