import { createPinia } from 'pinia';

export const pinia = createPinia();

export * from './modules/accounts';
export * from './modules/settings';
export * from './modules/ui';
