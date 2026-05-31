import { authApi, api } from './client'
import type { LoginRequest, RegisterRequest, AuthResponse, User } from '@/types'

export const authService = {
  login: (data: LoginRequest) => api.post<AuthResponse>(authApi, '/api/auth/login', data),

  register: (data: RegisterRequest) => api.post<AuthResponse>(authApi, '/api/auth/register', data),

  refresh: (refreshToken: string) =>
    api.post<AuthResponse>(authApi, '/api/auth/refresh', { refresh_token: refreshToken }),

  logout: () => api.post<null>(authApi, '/api/auth/logout', {}),

  getUsers: () => api.get<User[]>(authApi, '/api/users'),

  getUser: (id: number) => api.get<User>(authApi, `/api/users/${id}`),
}
