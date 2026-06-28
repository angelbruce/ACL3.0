import vecClient from './client';
import type { Version, VersionDiff } from '@/vec/types/version';

export const versionApi = {
  list: async (documentId: string) => {
    const res = await vecClient.get<{ code: number; data?: Version[] }>(`/documents/${documentId}/versions`);
    return res.data || [];
  },

  create: async (documentId: string, content?: string, changeNote?: string) => {
    await vecClient.post(`/documents/${documentId}/versions`, { content, change_note: changeNote });
  },

  get: async (versionId: string) => {
    const res = await vecClient.get<{ code: number; data?: Version }>(`/versions/${versionId}`);
    return res.data;
  },

  rollback: async (documentId: string, versionId: number) => {
    await vecClient.post(`/documents/${documentId}/rollback`, { version_id: versionId });
  },

  diff: async (versionA: number, versionB: number) => {
    const res = await vecClient.get<{ code: number; data?: VersionDiff }>('/versions/compare', { params: { version_a: versionA, version_b: versionB } });
    return res.data;
  },
};
