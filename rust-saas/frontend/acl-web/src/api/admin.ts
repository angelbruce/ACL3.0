import axios, { AxiosInstance } from 'axios'
import { api } from './client'

const ADMIN_API_BASE = import.meta.env.VITE_API_ADMIN_URL || 
  (window.location.host === 'localhost:8086' ? 'http://localhost:8086' : `http://${window.location.host}/foreign/admin`)

const adminApi: AxiosInstance = axios.create({
  baseURL: ADMIN_API_BASE,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

adminApi.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('access_token')
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => Promise.reject(error)
)

import type {
  Personnel,
  Department,
  Menu,
  Permission,
  Role,
  PersonnelWithDetails,
  CreatePersonnelRequest,
  UpdatePersonnelRequest,
  CreateDepartmentRequest,
  UpdateDepartmentRequest,
  CreateMenuRequest,
  UpdateMenuRequest,
  CreatePermissionRequest,
  CreateRoleRequest,
  UpdateRoleRequest,
} from '@/types/admin'

export const adminService = {
  initSuperAdmin: () => api.post<Role>(adminApi, '/init-super-admin', {}),
  initMenus: () => api.post<Menu[]>(adminApi, '/init-menus', {}),
  initPermissions: () => api.post<Permission[]>(adminApi, '/init-permissions', {}),
  initSuperAdminAll: () => api.post(adminApi, '/init-super-admin-all', {}),

  getPersonnelList: () => api.get<Personnel[]>(adminApi, '/personnel'),
  getPersonnel: (id: number) => api.get<Personnel>(adminApi, `/personnel/${id}`),
  getPersonnelDetails: (id: number) => api.get<PersonnelWithDetails>(adminApi, `/personnel/${id}/details`),
  createPersonnel: (data: CreatePersonnelRequest) =>
    api.post<Personnel>(adminApi, '/personnel', data),
  updatePersonnel: (id: number, data: UpdatePersonnelRequest) =>
    api.put<Personnel>(adminApi, `/personnel/${id}`, data),
  assignSuperAdmin: (personnelId: number) =>
    api.post(adminApi, `/personnel/${personnelId}/assign-super-admin`, {}),

  getDepartmentList: () => api.get<Department[]>(adminApi, '/departments'),
  getDepartment: (id: number) => api.get<Department>(adminApi, `/departments/${id}`),
  createDepartment: (data: CreateDepartmentRequest) =>
    api.post<Department>(adminApi, '/departments', data),
  updateDepartment: (id: number, data: UpdateDepartmentRequest) =>
    api.put<Department>(adminApi, `/departments/${id}`, data),
  deleteDepartment: (id: number) => api.delete(adminApi, `/departments/${id}`),

  getMenuList: () => api.get<Menu[]>(adminApi, '/menus'),
  getMenu: (id: number) => api.get<Menu>(adminApi, `/menus/${id}`),
  createMenu: (data: CreateMenuRequest) => api.post<Menu>(adminApi, '/menus', data),
  updateMenu: (id: number, data: UpdateMenuRequest) =>
    api.put<Menu>(adminApi, `/menus/${id}`, data),
  deleteMenu: (id: number) => api.delete(adminApi, `/menus/${id}`),

  getPermissionList: () => api.get<Permission[]>(adminApi, '/permissions'),
  getPermissionsByMenu: (menuId: number) =>
    api.get<Permission[]>(adminApi, `/permissions/by-menu/${menuId}`),
  createPermission: (data: CreatePermissionRequest) =>
    api.post<Permission>(adminApi, '/permissions', data),
  deletePermission: (id: number) => api.delete(adminApi, `/permissions/${id}`),

  getRoleList: () => api.get<Role[]>(adminApi, '/roles'),
  getRole: (id: number) => api.get<Role>(adminApi, `/roles/${id}`),
  createRole: (data: CreateRoleRequest) => api.post<Role>(adminApi, '/roles', data),
  updateRole: (id: number, data: UpdateRoleRequest) =>
    api.put<Role>(adminApi, `/roles/${id}`, data),
  deleteRole: (id: number) => api.delete(adminApi, `/roles/${id}`),
  getRolePermissions: (roleId: number) =>
    api.get<Permission[]>(adminApi, `/roles/${roleId}/permissions`),
  assignPermissionsToRole: (roleId: number, permissionIds: number[]) =>
    api.post(adminApi, `/roles/${roleId}/permissions`, permissionIds),

  assignDepartments: (personnelId: number, departmentIds: number[]) =>
    api.post(adminApi, `/personnel/${personnelId}/assign-departments`, { personnel_id: personnelId, department_ids: departmentIds }),
  assignRoles: (personnelId: number, roleIds: number[]) =>
    api.post(adminApi, `/personnel/${personnelId}/assign-roles`, { personnel_id: personnelId, role_ids: roleIds }),
  getPersonnelDepartments: (personnelId: number) =>
    api.get<Department[]>(adminApi, `/personnel/${personnelId}/departments`),
  getPersonnelRoles: (personnelId: number) =>
    api.get<Role[]>(adminApi, `/personnel/${personnelId}/roles`),
  getPersonnelPermissions: (personnelId: number) =>
    api.get<Permission[]>(adminApi, `/personnel/${personnelId}/permissions`),
  checkSuperAdmin: (personnelId: number) =>
    api.get<boolean>(adminApi, `/personnel/${personnelId}/is-super-admin`),
}