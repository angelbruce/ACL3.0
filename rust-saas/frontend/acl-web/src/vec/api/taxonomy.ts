import vecClient from './client';
import type { Category, Level } from '@/vec/types/taxonomy';

export const taxonomyApi = {
  categories: async () => {
    const data = await vecClient.get<Category[]>('/categories');
    return data;
  },

  category: async (id: string) => {
    const data = await vecClient.get<Category>(`/categories/${id}`);
    return data;
  },

  createCategory: async (data: { name: string; parent_id?: string }) => {
    const result = await vecClient.post<Category>('/categories', data);
    return result;
  },

  updateCategory: async (id: string, data: { name?: string; parent_id?: string }) => {
    const result = await vecClient.put<Category>(`/categories/${id}`, data);
    return result;
  },

  deleteCategory: async (id: string) => {
    await vecClient.delete(`/categories/${id}`);
  },

  childCategories: async (parentId: string) => {
    const data = await vecClient.get<Category[]>(`/categories/${parentId}/children`);
    return data;
  },

  documentCategories: async (documentId: string) => {
    const data = await vecClient.get<Category[]>(`/documents/${documentId}/categories`);
    return data;
  },

  assignDocumentCategories: async (documentId: string, categoryIds: string[]) => {
    await vecClient.post(`/documents/${documentId}/categories`, { category_ids: categoryIds });
  },

  levels: async () => {
    const data = await vecClient.get<Level[]>('/levels');
    return data;
  },

  level: async (id: string) => {
    const data = await vecClient.get<Level>(`/levels/${id}`);
    return data;
  },

  createLevel: async (data: { name: string; value: string }) => {
    const result = await vecClient.post<Level>('/levels', data);
    return result;
  },

  updateLevel: async (id: string, data: { name?: string; value?: string }) => {
    const result = await vecClient.put<Level>(`/levels/${id}`, data);
    return result;
  },

  deleteLevel: async (id: string) => {
    await vecClient.delete(`/levels/${id}`);
  },

  documentLevels: async (documentId: string) => {
    const data = await vecClient.get<Level[]>(`/documents/${documentId}/levels`);
    return data;
  },

  assignDocumentLevels: async (documentId: string, levelIds: string[]) => {
    await vecClient.post(`/documents/${documentId}/levels`, { level_ids: levelIds });
  },
};
