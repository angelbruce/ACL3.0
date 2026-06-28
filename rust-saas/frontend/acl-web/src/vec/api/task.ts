import vecClient from './client';
import type { Task } from '@/vec/types/task';

export const taskApi = {
  list: async (params?: { document_id?: string; status?: string }) => {
    const data = await vecClient.get<{ tasks: Task[] }>('/tasks', { params });
    return data;
  },

  get: async (id: string) => {
    const data = await vecClient.get<Task>(`/tasks/${id}`);
    return data;
  },

  cancel: async (id: string) => {
    await vecClient.delete(`/tasks/${id}`);
  },
};
