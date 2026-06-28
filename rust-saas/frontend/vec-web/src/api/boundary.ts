import apiClient from './client';
import type { Share } from '@/types/boundary';

export const boundaryApi = {
  shares: async (documentId: string) => {
    const data = await apiClient.get<Share[]>('/boundary/shares', { params: { document_id: documentId } });
    return data;
  },

  share: async (data: { document_id: string; user_id?: string; group_id?: string; permission: string }) => {
    const result = await apiClient.post<Share>('/boundary/share', data);
    return result;
  },

  deleteShare: async (id: string) => {
    await apiClient.delete(`/boundary/shares/${id}`);
  },

  setVisibility: async (documentId: string, data: { visibility: 'public' | 'private' | 'restricted' }) => {
    await apiClient.put(`/boundary/visibility`, { document_id: documentId, ...data });
  },

  permissions: async (documentId: string) => {
    const data = await apiClient.get<{ read: boolean; write: boolean; share: boolean }>('/boundary/permissions', { params: { document_id: documentId } });
    return data;
  },
};
