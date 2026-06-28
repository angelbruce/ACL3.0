import vecClient from './client';
import type { ImportResult, KnowledgeGraphExport } from '@/vec/types';

export const importExportApi = {
  importDocuments: async (files: File[]) => {
    const formData = new FormData();
    files.forEach((file) => formData.append('files', file));
    const data = await vecClient.post<ImportResult>('/import/documents', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
    return data;
  },

  exportDocuments: async (ids?: string[]) => {
    const data = await vecClient.get<Blob>('/export/documents', {
      params: ids ? { ids: ids.join(',') } : {},
      responseType: 'blob',
    });
    return data;
  },

  exportKnowledgeGraph: async () => {
    const data = await vecClient.get<KnowledgeGraphExport>('/export/knowledge-graph');
    return data;
  },
};
