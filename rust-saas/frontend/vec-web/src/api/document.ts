import apiClient from './client';
import type { Document, Chunk, KnowledgePoint } from '@/types/document';
import type { PaginatedResponse } from '@/types/common';

export const documentApi = {
  list: async (params?: { page?: number; page_size?: number; category_id?: string; status?: string; visibility?: string }) => {
    const data = await apiClient.get<PaginatedResponse<Document>>('/documents', { params });
    return data;
  },

  get: async (id: string) => {
    const data = await apiClient.get<Document>(`/documents/${id}`);
    return data;
  },

  create: async (data: { title: string; content: string; document_type?: string; visibility?: string }) => {
    const result = await apiClient.post<Document>('/documents', data);
    return result;
  },

  update: async (id: string, data: { title?: string; content?: string; visibility?: string }) => {
    const result = await apiClient.put<Document>(`/documents/${id}`, data);
    return result;
  },

  delete: async (id: string) => {
    await apiClient.delete(`/documents/${id}`);
  },

  chunks: async (id: string) => {
    const data = await apiClient.get<Chunk[]>(`/documents/${id}/chunks`);
    return data;
  },

  getKnowledgePoints: async (id: string) => {
    const data = await apiClient.get<KnowledgePoint[]>(`/documents/${id}/knowledge-points`);
    return data;
  },

  distill: async (id: string) => {
    await apiClient.post(`/documents/${id}/distill`);
  },

  versions: async (id: string) => {
    const data = await apiClient.get<{ versions: { version: number; created_at: string }[] }>(`/documents/${id}/versions`);
    return data;
  },
};
