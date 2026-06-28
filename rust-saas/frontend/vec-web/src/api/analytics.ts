import apiClient from './client';
import type { AnalyticsSummary, DocumentAnalytics } from '@/types/analytics';

export const analyticsApi = {
  summary: async () => {
    const data = await apiClient.get<AnalyticsSummary>('/analytics/summary');
    return data;
  },

  document: async (documentId: string) => {
    const data = await apiClient.get<DocumentAnalytics>('/analytics/document', { params: { document_id: documentId } });
    return data;
  },
};
