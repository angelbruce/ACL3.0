export interface Personnel {
  id: number
  user_id: number
  name: string
  gender?: string
  email?: string
  wechat?: string
  phone?: string
  last_login_date?: string
  created_at: string
  updated_at: string
}

export interface Department {
  id: number
  name: string
  parent_id?: number
  description?: string
  created_at: string
  updated_at: string
}

export interface Menu {
  id: number
  name: string
  path?: string
  parent_id?: number
  icon?: string
  sort_order: number
  created_at: string
  updated_at: string
}

export interface Permission {
  id: number
  menu_id: number
  name: string
  description?: string
  created_at: string
}

export interface Role {
  id: number
  name: string
  description?: string
  is_super_admin: boolean
  created_at: string
  updated_at: string
}

export interface PersonnelWithDetails {
  personnel: Personnel
  departments: Department[]
  roles: Role[]
  permissions: Permission[]
}

export interface CreatePersonnelRequest {
  name: string
  user_id?: number
  gender?: string
  email?: string
  wechat?: string
  phone?: string
}

export interface UpdatePersonnelRequest {
  name?: string
  gender?: string
  email?: string
  wechat?: string
  phone?: string
}

export interface CreateDepartmentRequest {
  name: string
  parent_id?: number
  description?: string
}

export interface UpdateDepartmentRequest {
  name?: string
  parent_id?: number
  description?: string
}

export interface CreateMenuRequest {
  name: string
  path?: string
  parent_id?: number
  icon?: string
  sort_order?: number
}

export interface UpdateMenuRequest {
  name?: string
  path?: string
  parent_id?: number
  icon?: string
  sort_order?: number
}

export interface CreatePermissionRequest {
  menu_id: number
  name: string
  description?: string
}

export interface CreateRoleRequest {
  name: string
  description?: string
  is_super_admin?: boolean
  permission_ids?: number[]
}

export interface UpdateRoleRequest {
  name?: string
  description?: string
  is_super_admin?: boolean
}

export interface AssignRolesRequest {
  personnel_id: number
  role_ids: number[]
}

export interface AssignDepartmentsRequest {
  personnel_id: number
  department_ids: number[]
}
