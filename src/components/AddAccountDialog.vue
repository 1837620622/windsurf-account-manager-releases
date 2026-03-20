<template>
  <el-dialog
    v-model="uiStore.showAddAccountDialog"
    title="添加账号"
    width="500px"
    :close-on-click-modal="false"
  >
    <el-form
      ref="formRef"
      :model="formData"
      :rules="currentRules"
      label-width="100px"
      autocomplete="off"
    >
      <!-- 添加方式切换 -->
      <el-form-item label="添加方式">
        <el-radio-group v-model="addMode" @change="handleModeChange">
          <el-radio value="password">邮箱密码</el-radio>
          <el-radio value="refresh_token">Refresh Token</el-radio>
        </el-radio-group>
      </el-form-item>

      <!-- 邮箱密码模式 -->
      <template v-if="addMode === 'password'">
        <el-form-item label="邮箱" prop="email">
          <el-input
            v-model="formData.email"
            placeholder="请输入邮箱"
            :prefix-icon="Message"
            autocomplete="off"
          />
        </el-form-item>
        
        <el-form-item label="密码" prop="password">
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="请输入密码"
            :prefix-icon="Lock"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </template>

      <!-- Refresh Token 模式 -->
      <template v-else>
        <el-form-item label="Refresh Token" prop="refreshToken">
          <el-input
            v-model="formData.refreshToken"
            type="textarea"
            :rows="3"
            placeholder="请输入 Refresh Token"
          />
        </el-form-item>
      </template>
      
      <el-form-item label="备注名称" prop="nickname">
        <el-input
          v-model="formData.nickname"
          placeholder="留空则使用邮箱用户名"
          :prefix-icon="User"
        />
      </el-form-item>
      
      <el-form-item label="分组">
        <el-select
          v-model="formData.group"
          placeholder="选择分组"
          clearable
        >
          <el-option
            v-for="group in settingsStore.groups"
            :key="group"
            :label="group"
            :value="group"
          />
        </el-select>
      </el-form-item>
      
      <el-form-item label="标签">
        <el-select
          v-model="formData.tags"
          multiple
          filterable
          allow-create
          placeholder="输入或选择标签"
          style="width: 100%"
        >
          <el-option
            v-for="tag in settingsStore.tags"
            :key="tag.name"
            :label="tag.name"
            :value="tag.name"
          >
            <span :style="getTagOptionStyle(tag.color)">{{ tag.name }}</span>
          </el-option>
        </el-select>
      </el-form-item>
    </el-form>
    
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="loading">
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { ElMessage } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { Message, Lock, User } from '@element-plus/icons-vue';
import { useAccountsStore, useSettingsStore, useUIStore } from '@/store';
import { apiService, accountApi } from '@/api';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAccountsStore();
const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const formRef = ref<FormInstance>();
const loading = ref(false);
const addMode = ref<'password' | 'refresh_token'>('password');

const formData = reactive({
  email: '',
  password: '',
  refreshToken: '',
  nickname: '',
  group: '默认分组',
  tags: [] as string[]
});

// 邮箱密码模式的验证规则
const passwordRules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少6位', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: '备注名称最多20个字符', trigger: 'blur' }
  ]
};

// Refresh Token 模式的验证规则
const refreshTokenRules: FormRules = {
  refreshToken: [
    { required: true, message: '请输入 Refresh Token', trigger: 'blur' },
    { min: 10, message: 'Refresh Token 格式不正确', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: '备注名称最多20个字符', trigger: 'blur' }
  ]
};

// 根据模式选择验证规则
const currentRules = computed(() => {
  return addMode.value === 'password' ? passwordRules : refreshTokenRules;
});

// 切换模式时重置表单
function handleModeChange() {
  formRef.value?.resetFields();
}

// 获取标签选项样式
function getTagOptionStyle(color: string): Record<string, string> {
  if (!color) return {};
  
  let r = 0, g = 0, b = 0;
  let parsed = false;
  
  // 解析 rgba 或 rgb 格式
  if (color.startsWith('rgba') || color.startsWith('rgb')) {
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
    if (match) {
      r = parseInt(match[1]);
      g = parseInt(match[2]);
      b = parseInt(match[3]);
      parsed = true;
    }
  } 
  // 解析 HEX 格式
  if (!parsed && color.startsWith('#')) {
    const hex = color.slice(1);
    if (hex.length >= 6) {
      r = parseInt(hex.slice(0, 2), 16);
      g = parseInt(hex.slice(2, 4), 16);
      b = parseInt(hex.slice(4, 6), 16);
      parsed = true;
    }
  }
  
  if (!parsed) return {};
  
  return {
    color: `rgb(${r}, ${g}, ${b})`,
    fontWeight: '500'
  };
}

async function handleSubmit() {
  if (!formRef.value) return;
  
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    
    loading.value = true;
    try {
      if (addMode.value === 'refresh_token') {
        // Refresh Token 模式
        const trimmedToken = formData.refreshToken.trim();
        const trimmedNickname = formData.nickname.trim() || undefined;
        
        if (!trimmedToken) {
          ElMessage.error('Refresh Token 不能为空');
          loading.value = false;
          return;
        }
        
        // 调用后端接口添加账号
        const result = await invoke<any>('add_account_by_refresh_token', {
          refreshToken: trimmedToken,
          nickname: trimmedNickname,
          tags: formData.tags,
          group: formData.group || '默认分组'
        });
        
        if (result.success) {
          ElMessage.success(`账号 ${result.email} 添加成功`);
          // 刷新账号列表
          await accountsStore.loadAccounts();
          handleClose();
        } else {
          ElMessage.error(result.error || '添加失败');
        }
      } else {
        // 邮箱密码模式
        const trimmedEmail = formData.email.trim();
        const trimmedPassword = formData.password.trim();
        const trimmedNickname = formData.nickname.trim() || trimmedEmail.split('@')[0];
        
        if (!trimmedPassword) {
          ElMessage.error('密码不能为空或只包含空格');
          loading.value = false;
          return;
        }
        
        // 添加账号
        const newAccount = await accountsStore.addAccount({
          email: trimmedEmail,
          password: trimmedPassword,
          nickname: trimmedNickname,
          tags: formData.tags,
          group: formData.group || '默认分组'
        });
        
        ElMessage.success('账号添加成功，正在获取账号信息...');
        
        // 自动登录并获取账号详细信息
        try {
          const loginResult = await apiService.loginAccount(newAccount.id);
          
          if (loginResult.success) {
            const latestAccount = await accountApi.getAccount(newAccount.id);
            await accountsStore.updateAccount(latestAccount);
            ElMessage.success('账号信息已更新');
          } else {
            ElMessage.warning('账号已添加，但登录失败，请手动刷新');
          }
        } catch (infoError) {
          console.error('获取账号信息失败:', infoError);
          ElMessage.warning('账号已添加，但获取详细信息失败，请手动刷新');
        }
        
        handleClose();
      }
    } catch (error) {
      ElMessage.error(`添加失败: ${error}`);
    } finally {
      loading.value = false;
    }
  });
}

function handleClose() {
  uiStore.closeAddAccountDialog();
  formRef.value?.resetFields();
  
  // 重置表单数据
  formData.email = '';
  formData.password = '';
  formData.refreshToken = '';
  formData.nickname = '';
  formData.group = '默认分组';
  formData.tags = [];
  addMode.value = 'password';
}
</script>
