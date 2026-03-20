<template>
  <el-dialog
    v-model="visible"
    title="座位更新结果"
    width="600px"
    @close="handleClose"
  >
    <div v-if="resultData">
      <!-- 主要信息卡片 -->
      <el-card v-if="resultData.success" class="result-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>更新成功</span>
            <el-tag type="success">
              <el-icon><Check /></el-icon>
              座位数: {{ lastAttempt?.total_seats || 'N/A' }}
            </el-tag>
          </div>
        </template>
        
        <!-- 座位信息 -->
        <div class="seats-section" v-if="lastAttempt">
          <el-descriptions :column="2" border>
            <el-descriptions-item label="总座位数">
              <el-text type="success" style="font-size: 16px; font-weight: bold;">{{ lastAttempt.total_seats || 0 }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="计费周期">
              <el-text type="primary">{{ lastAttempt.billing_interval === 'yearly' ? '年付' : '月付' }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="每座位价格">
              <el-text type="info">${{ lastAttempt.price_per_seat || 0 }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="总费用">
              <el-text type="warning" style="font-size: 16px; font-weight: bold;">${{ lastAttempt.total_monthly_price || 0 }}</el-text>
            </el-descriptions-item>
          </el-descriptions>
          
          <!-- 立即应付金额（如果有） -->
          <div v-if="lastAttempt.amount_due_immediately > 0" style="margin-top: 15px;">
            <el-alert
              :title="`立即应付: $${lastAttempt.amount_due_immediately}`"
              type="warning"
              :closable="false"
              show-icon
            />
          </div>
        </div>
        
        <!-- 时间信息 -->
        <div class="time-section" v-if="lastAttempt && (lastAttempt.billing_start_time || lastAttempt.next_billing_time)">
          <h4>计费周期</h4>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="当前周期开始" v-if="lastAttempt.billing_start_time">
              <el-text>{{ lastAttempt.billing_start_time }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="下次计费时间" v-if="lastAttempt.next_billing_time">
              <el-text type="warning">{{ lastAttempt.next_billing_time }}</el-text>
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </el-card>
      
      <!-- 错误信息 -->
      <el-alert
        v-if="!resultData.success"
        title="座位更新失败"
        :description="getErrorMessage()"
        type="error"
        :closable="false"
        show-icon
      />
      
      <!-- 尝试记录 -->
      <div class="attempts-section" v-if="resultData.attempts?.length > 0">
        <h4>尝试记录 ({{ resultData.attempts.length }}次)</h4>
        <el-collapse>
          <el-collapse-item 
            v-for="(attempt, index) in resultData.attempts" 
            :key="index"
            :title="`尝试 #${attempt.attempt} - 状态码: ${attempt.status_code || 'N/A'}`"
          >
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="时间">{{ attempt.timestamp }}</el-descriptions-item>
              <el-descriptions-item label="状态码">{{ attempt.status_code || 'N/A' }}</el-descriptions-item>
              <el-descriptions-item label="错误" v-if="attempt.error">
                <el-text type="danger">{{ attempt.error }}</el-text>
              </el-descriptions-item>
            </el-descriptions>
            <div v-if="attempt.raw_response" style="margin-top: 10px;">
              <el-text type="info" size="small">响应数据:</el-text>
              <pre class="raw-response">{{ formatResponse(attempt.raw_response) }}</pre>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
    </div>
    
    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
      <el-button type="primary" @click="copyToClipboard" v-if="resultData">
        复制数据
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { Check } from '@element-plus/icons-vue';

const props = defineProps<{
  modelValue: boolean;
  resultData?: any;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const visible = ref(props.modelValue);

watch(() => props.modelValue, (val) => {
  visible.value = val;
});

watch(visible, (val) => {
  emit('update:modelValue', val);
});

// 获取最后一次尝试的解析数据
const lastAttempt = computed(() => {
  if (!props.resultData?.attempts?.length) return null;
  const last = props.resultData.attempts[props.resultData.attempts.length - 1];
  if (last?.raw_response) {
    try {
      return JSON.parse(last.raw_response);
    } catch {
      return null;
    }
  }
  return null;
});


// 获取错误信息
function getErrorMessage() {
  if (!props.resultData?.attempts?.length) return '未知错误';
  const lastAttempt = props.resultData.attempts[props.resultData.attempts.length - 1];
  return lastAttempt?.error || `HTTP ${lastAttempt?.status_code || '未知状态码'}`;
}

// 格式化响应数据
function formatResponse(response: string) {
  try {
    const parsed = JSON.parse(response);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return response;
  }
}

function handleClose() {
  visible.value = false;
}

async function copyToClipboard() {
  if (props.resultData) {
    try {
      await navigator.clipboard.writeText(JSON.stringify(props.resultData, null, 2));
      ElMessage.success('已复制到剪贴板');
    } catch (error) {
      ElMessage.error('复制失败');
    }
  }
}
</script>

<style scoped>
.result-card {
  border: none;
  margin-bottom: 20px;
}

.result-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 16px;
  font-weight: 600;
}

.seats-section {
  margin-bottom: 20px;
}

.time-section {
  margin-top: 20px;
}

.time-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #606266;
}

.attempts-section {
  margin-top: 20px;
}

.attempts-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #606266;
}

.raw-response {
  margin-top: 5px;
  padding: 10px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

/* 暗色主题支持 */
:root.dark .result-card {
  background: #1e1e1e;
}

:root.dark .raw-response {
  background: #2a2a2a;
  color: #e4e4e7;
}
</style>
