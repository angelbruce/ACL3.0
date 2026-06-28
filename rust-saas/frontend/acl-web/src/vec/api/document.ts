import vecClient from './client';
import type { Document, KnowledgePoint } from '@/vec/types/document';
import type { PaginatedResponse } from '@/vec/types/common';

export const documentApi = {
  list: async (params?: { page?: number; page_size?: number; project_id?: number; status?: string }) => {
    const data = await vecClient.get<PaginatedResponse<Document>>('/documents', { params });
    return data;
  },

  get: async (id: string) => {
    const data = await vecClient.get<Document>(`/documents/${id}`);
    return data;
  },

  createText: async (data: { topic?: string; content: string; project_id?: number; metadata?: Record<string, unknown> }) => {
    const result = await vecClient.post<Document>('/documents/text', data);
    return result;
  },

  uploadFile: async (file: File, topic?: string, project_id?: number) => {
    const formData = new FormData();
    formData.append('file', file);
    if (topic) formData.append('topic', topic);
    if (project_id !== undefined) formData.append('project_id', String(project_id));
    const result = await vecClient.post<Document>('/documents/file', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return result;
  },

  delete: async (id: string) => {
    await vecClient.delete(`/documents/${id}`);
  },

  reindex: async (id: string) => {
    const result = await vecClient.post<{ success: boolean; id: number; message?: string }>(`/documents/${id}/reindex`);
    return result;
  },

  distill: async (id: string) => {
    const data = await vecClient.post<KnowledgePoint[]>(`/documents/${id}/distill`, {});
    return data;
  },

  getKnowledgePoints: async (id: string) => {
    const data = await vecClient.get<KnowledgePoint[]>(`/documents/${id}/knowledge-points`);
    return data;
  },

  versions: async (id: string) => {
    const data = await vecClient.get<{ versions: { version: number; created_at: string }[] }>(`/documents/${id}/versions`);
    return data;
  },
};
