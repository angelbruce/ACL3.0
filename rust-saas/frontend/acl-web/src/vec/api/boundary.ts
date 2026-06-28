import vecClient from './client';
import type { Share } from '@/vec/types/boundary';

export const boundaryApi = {
  shares: async (documentId: string) => {
    const data = await vecClient.get<Share[]>(`/documents/${documentId}/shares`);
    return data;
  },

  share: async (data: { document_id: string; user_id?: string; group_id?: string; permission: string }) => {
    const result = await vecClient.post<Share>('/shares', data);
    return result;
  },

  deleteShare: async (id: string) => {
    await vecClient.delete(`/shares/${id}`);
  },

  setVisibility: async (documentId: string, data: { visibility: 'public' | 'private' | 'restricted' }) => {
    await vecClient.post(`/documents/${documentId}/visibility`, data);
  },

  boundary: async (documentId: string) => {
    const data = await vecClient.get<{ boundary_type?: string; owner_id?: number; project_id?: number }>(`/documents/${documentId}/boundary`);
    return data;
  },

  checkAccess: async (documentId: string, userId?: string) => {
    const data = await vecClient.post<{ has_access: boolean }>(`/documents/${documentId}/check-access`, { user_id: userId });
    return data;
  },

  checkShareAccess: async (documentId: string, token: string) => {
    const data = await vecClient.post<{ has_access: boolean }>(`/documents/${documentId}/check-share`, { token });
    return data;
  },

  accessibleDocuments: async (userId?: string) => {
    const data = await vecClient.post<number[]>('/documents/accessible', { user_id: userId });
    return data;
  },
};
