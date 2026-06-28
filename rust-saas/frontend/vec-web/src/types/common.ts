export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface ApiResponse<T = unknown> {
  success: boolean;
  message?: string;
  data?: T;
  error?: string;
}

export interface BaseEntity {
  id: string;
  created_at: string;
  updated_at: string;
}

export type Status = 'pending' | 'processing' | 'completed' | 'failed';

export type Visibility = 'public' | 'private' | 'restricted';

export type DocumentType = 'text' | 'pdf' | 'doc' | 'docx' | 'ppt' | 'pptx' | 'image' | 'audio' | 'video';
