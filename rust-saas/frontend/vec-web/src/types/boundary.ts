import type { BaseEntity, Visibility } from './common';

export interface Share extends BaseEntity {
  document_id: string;
  user_id?: string;
  group_id?: string;
  permission: 'read' | 'write';
}

export interface VisibilityUpdate {
  visibility: Visibility;
  allowed_users?: string[];
  allowed_groups?: string[];
}
