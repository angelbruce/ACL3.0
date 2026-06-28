import vecClient from './client';
import type { GraphData, Entity, Relation } from '@/vec/types/graph';

export const graphApi = {
  data: async () => {
    const data = await vecClient.get<Entity[]>('/graph/entities');
    return data;
  },

  entities: async (params?: { entity_type?: string; limit?: number }) => {
    const data = await vecClient.get<Entity[]>('/graph/entities', { params });
    return data;
  },

  entity: async (id: string) => {
    const data = await vecClient.get<Entity>(`/graph/entities/${id}`);
    return data;
  },

  entityRelations: async (id: string) => {
    const data = await vecClient.get<Relation[]>(`/graph/entities/${id}/relations`);
    return data;
  },

  projectEntities: async (projectId: number, params?: { entity_type?: string }) => {
    const data = await vecClient.get<Entity[]>(`/graph/projects/${projectId}/entities`, { params });
    return data;
  },

  extract: async (documentId: string) => {
    const data = await vecClient.post<GraphData>('/graph/extract', { document_id: documentId });
    return data;
  },
};
