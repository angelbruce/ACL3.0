import apiClient from './client';
import type { GraphData, Entity, Relation } from '@/types/graph';

export const graphApi = {
  data: async () => {
    const data = await apiClient.get<GraphData>('/knowledge-graph');
    return data;
  },

  entities: async (params?: { entity_type?: string; limit?: number }) => {
    const data = await apiClient.get<Entity[]>('/knowledge-graph/entities', { params });
    return data;
  },

  entity: async (id: string) => {
    const data = await apiClient.get<Entity>(`/knowledge-graph/entities/${id}`);
    return data;
  },

  relations: async () => {
    const data = await apiClient.get<Relation[]>('/knowledge-graph/relations');
    return data;
  },

  subgraph: async (entityId: string, depth?: number) => {
    const data = await apiClient.get<GraphData>('/knowledge-graph/subgraph', { params: { entity_id: entityId, depth } });
    return data;
  },
};
