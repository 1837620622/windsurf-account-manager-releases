<template>
  <el-dialog
    v-model="uiStore.showEditAccountDialog"
    title="编辑账号"
    width="500px"
    :close-on-click-modal="false"
  >
    <el-form
      v-if="currentAccount"
      ref="formRef"
      :model="formData"
      :rules="rules"
      label-width="100px"
      autocomplete="off"
    >
      <el-form-item label="邮箱">
        <el-input
          v-model="formData.email"
          disabled
          :prefix-icon="Message"
          autocomplete="off"
        />
      </el-form-item>
      
      <el-form-item label="备注名称" prop="nickname">
        <el-input
          v-model="formData.nickname"
          placeholder="请输入备注名称"
          :prefix-icon="User"
        />
      </el-form-item>
      
      <el-form-item label="修改密码" prop="newPassword">
        <el-input 
          v-model="formData.newPassword" 
          type="password"
          placeholder="留空则不修改密码"
          show-password
          autocomplete="new-password"
        />
      </el-form-item>
      
      <el-form-item label="确认密码" prop="confirmPassword" v-if="formData.newPassword">
        <el-input 
          v-model="formData.confirmPassword" 
          type="password"
          placeholder="请再次输入密码"
          show-password
          autocomplete="new-password"
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
          @change="handleTagsChange"
        >
          <el-option
            v-for="tag in availableTags"
            :key="tag"
            :label="tag"
            :value="tag"
          >
            <span class="tag-option">
              <span 
                class="tag-color-dot" 
                :style="{ backgroundColor: getGlobalTagColor(tag) || '#909399' }"
              ></span>
              <span>{{ tag }}</span>
            </span>
          </el-option>
        </el-select>
      </el-form-item>
      
      <el-form-item label="标签颜色" v-if="formData.tags.length > 0">
        <TagColorPicker
          :tags="formData.tags"
          v-model:tagColors="formData.tagColors"
        />
      </el-form-item>
    </el-form>
    
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="loading">
        保存
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { ElMessage } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { Message, User } from '@element-plus/icons-vue';
import { useAccountsStore, useSettingsStore, useUIStore } from '@/store';
import type { Account, TagWithColor } from '@/types';
import TagColorPicker from '@/components/TagColorPicker.vue';

const accountsStore = useAccountsStore();
const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const formRef = ref<FormInstance>();
const loading = ref(false);

const formData = reactive({
  email: '',
  nickname: '',
  newPassword: '',
  confirmPassword: '',
  group: '',
  tags: [] as string[],
  tagColors: [] as TagWithColor[]
});

const currentAccount = computed(() => {
  if (!uiStore.currentEditingAccountId) return null;
  return accountsStore.accounts.find(a => a.id === uiStore.currentEditingAccountId);
});

watch(currentAccount, (account) => {
  if (account) {
    formData.email = account.email;
    formData.nickname = account.nickname;
    formData.newPassword = '';
    formData.confirmPassword = '';
    formData.group = account.group || '';
    formData.tags = [...account.tags];
    formData.tagColors = account.tagColors ? [...account.tagColors] : [];
  }
});

const validatePassword = (_rule: any, value: any, callback: any) => {
  if (value && formData.newPassword && value !== formData.newPassword) {
    callback(new Error('两次输入密码不一致'));
  } else {
    callback();
  }
};

const rules: FormRules = {
  nickname: [
    { required: true, message: '请输入备注名称', trigger: 'blur' },
    { max: 20, message: '备注名称最多20个字符', trigger: 'blur' }
  ],
  newPassword: [
    { min: 6, message: '密码长度至少6位', trigger: 'blur' }
  ],
  confirmPassword: [
    { validator: validatePassword, trigger: 'blur' }
  ]
};

const availableTags = computed(() => {
  const tags = new Set<string>();
  // 添加全局标签
  settingsStore.tags.forEach(tag => tags.add(tag.name));
  // 添加账号中已使用的标签
  accountsStore.accounts.forEach(account => {
    account.tags.forEach(tag => tags.add(tag));
  });
  return Array.from(tags);
});

// 获取全局标签的颜色
function getGlobalTagColor(tagName: string): string | undefined {
  const globalTag = settingsStore.tags.find(t => t.name === tagName);
  return globalTag?.color;
}

// 当标签列表变化时，自动应用全局标签的默认颜色
function handleTagsChange(newTags: string[]) {
  newTags.forEach(tagName => {
    // 如果该标签还没有颜色配置，且存在全局标签颜色，则自动应用
    const hasColor = formData.tagColors.some(tc => tc.name === tagName);
    if (!hasColor) {
      const globalColor = getGlobalTagColor(tagName);
      if (globalColor) {
        formData.tagColors.push({ name: tagName, color: globalColor });
      }
    }
  });
  // 移除不存在的标签的颜色
  formData.tagColors = formData.tagColors.filter(tc => newTags.includes(tc.name));
}

async function handleSubmit() {
  if (!formRef.value || !currentAccount.value) return;
  
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    
    // 在回调中再次获取currentAccount以满足TypeScript的类型检查
    const account = currentAccount.value;
    if (!account) return;
    
    loading.value = true;
    try {
      const updatedAccount: Account = {
        ...account,
        nickname: formData.nickname.trim(),
        tags: formData.tags,
        tagColors: formData.tagColors.filter(tc => formData.tags.includes(tc.name)),
        group: formData.group || undefined
      };
      
      // 如果输入了新密码，添加密码字段（去除首尾空格）
      if (formData.newPassword) {
        const trimmedPassword = formData.newPassword.trim();
        if (!trimmedPassword) {
          ElMessage.error('新密码不能为空或只包含空格');
          loading.value = false;
          return;
        }
        updatedAccount.password = trimmedPassword;
      } else {
        // 不修改密码时，确保不发送password字段
        delete updatedAccount.password;
      }
      
      await accountsStore.updateAccount(updatedAccount);
      
      ElMessage.success('账号更新成功');
      handleClose();
    } catch (error) {
      ElMessage.error(`更新失败: ${error}`);
    } finally {
      loading.value = false;
    }
  });
}

function handleClose() {
  uiStore.closeEditAccountDialog();
  formRef.value?.resetFields();
}
</script>

<style scoped>
/* 标签选项样式 */
.tag-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

/* 深色模式样式 */
:root.dark .el-form-item__label {
  color: #cfd3dc !important;
}

:root.dark .el-input__inner {
  background-color: #262729 !important;
  color: #cfd3dc !important;
}

:root.dark .el-select__input {
  color: #cfd3dc !important;
}

:root.dark .el-textarea__inner {
  background-color: #262729 !important;
  color: #cfd3dc !important;
  border-color: #4c4d4f !important;
}

:root.dark .el-select-dropdown {
  background-color: #1d1e1f !important;
  border-color: #4c4d4f !important;
}

:root.dark .el-select-dropdown__item {
  color: #cfd3dc !important;
}

:root.dark .el-select-dropdown__item:hover {
  background-color: #262729 !important;
}

:root.dark .el-tag {
  background-color: rgba(64, 158, 255, 0.1) !important;
  border-color: rgba(64, 158, 255, 0.3) !important;
  color: #409eff !important;
}
</style>
