import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosResponse } from 'axios';
import { ElMessage } from 'element-plus';

interface ApiClient extends AxiosInstance {
  get<T = unknown>(url: string, config?: AxiosRequestConfig): Promise<T>;
  post<T = unknown>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T>;
  put<T = unknown>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T>;
  delete<T = unknown>(url: string, config?: AxiosRequestConfig): Promise<T>;
}

const vecClient = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
}) as ApiClient;

vecClient.interceptors.response.use(
  (response: AxiosResponse) => {
    const data = response.data;
    if (data && typeof data === 'object') {
      if ('success' in data) {
        if (data.success) {
          return data.data;
        } else {
          ElMessage.error(data.message || '请求失败');
          throw new Error(data.message || '请求失败');
        }
      }
      if ('code' in data && data.code === 0 && 'data' in data) {
        return data.data;
      }
      if ('code' in data && data.code !== 0) {
        ElMessage.error(data.message || '请求失败');
        throw new Error(data.message || '请求失败');
      }
    }
    return data;
  },
  (error) => {
    const messageText = error.response?.data?.message || error.message || '网络错误';
    ElMessage.error(messageText);
    throw error;
  }
);

export default vecClient;
