import apiClient from './client';
import type { SearchResult, SearchSuggestion, AutocompleteItem } from '@/types/search';

export const searchApi = {
  query: async (query: string, params?: { limit?: number; category_id?: string }) => {
    const result = await apiClient.get<SearchResult[]>('/search', { params: { query, ...params } });
    return result;
  },

  suggestions: async (query: string, limit?: number) => {
    const data = await apiClient.get<SearchSuggestion[]>('/search/suggestions', { params: { query, limit } });
    return data;
  },

  autocomplete: async (query: string, limit?: number) => {
    const data = await apiClient.get<AutocompleteItem[]>('/search/autocomplete', { params: { query, limit } });
    return data;
  },

  related: async (query: string, limit?: number) => {
    const data = await apiClient.get<SearchResult[]>('/search/related', { params: { query, limit } });
    return data;
  },
};
