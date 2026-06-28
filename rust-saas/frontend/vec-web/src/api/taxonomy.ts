import apiClient from './client';
import type { Category, Level } from '@/types/taxonomy';

export const taxonomyApi = {
  categories: async () => {
    const data = await apiClient.get<Category[]>('/taxonomy/categories');
    return data;
  },

  category: async (id: string) => {
    const data = await apiClient.get<Category>(`/taxonomy/categories/${id}`);
    return data;
  },

  createCategory: async (data: { name: string; parent_id?: string }) => {
    const result = await apiClient.post<Category>('/taxonomy/categories', data);
    return result;
  },

  updateCategory: async (id: string, data: { name?: string; parent_id?: string }) => {
    const result = await apiClient.put<Category>(`/taxonomy/categories/${id}`, data);
    return result;
  },

  deleteCategory: async (id: string) => {
    await apiClient.delete(`/taxonomy/categories/${id}`);
  },

  categorizeDocument: async (documentId: string, categoryId: string) => {
    await apiClient.post('/taxonomy/categorize', { document_id: documentId, category_id: categoryId });
  },

  levels: async (categoryId: string) => {
    const data = await apiClient.get<Level[]>(`/taxonomy/levels`, { params: { category_id: categoryId } });
    return data;
  },

  level: async (id: string) => {
    const data = await apiClient.get<Level>(`/taxonomy/levels/${id}`);
    return data;
  },

  createLevel: async (data: { name: string; value: string; category_id: string }) => {
    const result = await apiClient.post<Level>('/taxonomy/levels', data);
    return result;
  },

  updateLevel: async (id: string, data: { name?: string; value?: string }) => {
    const result = await apiClient.put<Level>(`/taxonomy/levels/${id}`, data);
    return result;
  },

  deleteLevel: async (id: string) => {
    await apiClient.delete(`/taxonomy/levels/${id}`);
  },

  documentLevel: async (documentId: string) => {
    const data = await apiClient.get<{ level: number }>('/taxonomy/document-level', { params: { document_id: documentId } });
    return data;
  },

  setDocumentLevel: async (documentId: string, level: number) => {
    await apiClient.post('/taxonomy/set-document-level', { document_id: documentId, level });
  },
};
