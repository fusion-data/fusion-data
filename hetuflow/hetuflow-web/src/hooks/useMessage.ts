import { App } from 'antd';

/**
 * 使用 Ant Design App 组件提供的 message API
 * 替代静态方法调用，支持动态主题和上下文
 */
export const useMessage = () => {
  const { message } = App.useApp();
  return message;
};
