import type { BaseEntity, Status } from './common';

export interface Task extends BaseEntity {
  task_type: string;
  status: Status;
  progress: number;
  message?: string;
  result?: unknown;
  error?: string;
  document_id?: string;
}

export type TaskType = 'embedding' | 'distillation' | 'import' | 'export' | 'reindex';
