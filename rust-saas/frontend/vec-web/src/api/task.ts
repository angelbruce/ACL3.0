import apiClient from './client';
import type { Task } from '@/types/task';

export const taskApi = {
  list: async (params?: { document_id?: string; status?: string }) => {
    const data = await apiClient.get<Task[]>('/tasks', { params });
    return data;
  },

  get: async (id: string) => {
    const data = await apiClient.get<Task>(`/tasks/${id}`);
    return data;
  },

  cancel: async (id: string) => {
    await apiClient.delete(`/tasks/${id}`);
  },
};
