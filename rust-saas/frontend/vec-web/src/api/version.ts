import apiClient from './client';
import type { Version, VersionDiff } from '@/types/version';

export const versionApi = {
  list: async (documentId: string) => {
    const data = await apiClient.get<Version[]>(`/version/documents/${documentId}`);
    return data;
  },

  get: async (documentId: string, versionNumber: number) => {
    const data = await apiClient.get<Version>(`/version/documents/${documentId}/${versionNumber}`);
    return data;
  },

  rollback: async (documentId: string, versionNumber: number) => {
    await apiClient.post(`/version/rollback`, { document_id: documentId, version_number: versionNumber });
  },

  diff: async (documentId: string, versionA: number, versionB: number) => {
    const data = await apiClient.get<VersionDiff>(`/version/diff`, { params: { document_id: documentId, version_a: versionA, version_b: versionB } });
    return data;
  },
};
