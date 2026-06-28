import vecClient from './client';
import type { SearchResponse, SearchSuggestion, AutocompleteItem } from '@/vec/types/search';

export const searchApi = {
  query: async (query: string, params?: { project_id?: number; limit?: number; category_id?: string }) => {
    const result = await vecClient.post<SearchResponse>('/search', { query, top_k: params?.limit, ...params });
    return result;
  },

  projectSearch: async (projectId: number, query: string, params?: { limit?: number }) => {
    const result = await vecClient.get<SearchResponse>(`/projects/${projectId}/search`, { params: { query, top_k: params?.limit, ...params } });
    return result;
  },

  suggestions: async (query: string, limit?: number) => {
    const data = await vecClient.get<SearchSuggestion[]>('/search/suggest', { params: { query, limit } });
    return data;
  },

  autocomplete: async (query: string, limit?: number) => {
    const data = await vecClient.get<AutocompleteItem[]>('/search/autocomplete', { params: { query, limit } });
    return data;
  },
};
