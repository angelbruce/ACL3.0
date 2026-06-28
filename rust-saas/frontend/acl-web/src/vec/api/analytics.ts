import vecClient from './client';
import type { AnalyticsSummary, DocumentAnalytics } from '@/vec/types/analytics';

export const analyticsApi = {
  summary: async () => {
    const data = await vecClient.get<AnalyticsSummary>('/analytics/summary');
    return data;
  },

  document: async (documentId: string) => {
    const data = await vecClient.get<DocumentAnalytics>('/analytics/document', { params: { document_id: documentId } });
    return data;
  },
};
