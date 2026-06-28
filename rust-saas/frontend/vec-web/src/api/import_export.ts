import apiClient from './client';

export const importExportApi = {
  importDocuments: async (files: File[]) => {
    const formData = new FormData();
    files.forEach((file) => formData.append('files', file));
    await apiClient.post('/import/documents', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },

  exportDocuments: async (ids?: string[]) => {
    const data = await apiClient.get('/export/documents', {
      params: ids ? { ids: ids.join(',') } : {},
      responseType: 'blob',
    });
    return data;
  },

  exportKnowledgeGraph: async () => {
    const data = await apiClient.get('/export/knowledge-graph', { responseType: 'blob' });
    return data;
  },
};
