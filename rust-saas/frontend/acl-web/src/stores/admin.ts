import { defineStore } from 'pinia'
import { adminService } from '@/api'
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
} from '@/types'

export const useAdminStore = defineStore('admin', {
  state: () => ({
    personnel: [] as Personnel[],
    departments: [] as Department[],
    menus: [] as Menu[],
    permissions: [] as Permission[],
    roles: [] as Role[],
    currentPersonnel: null as PersonnelWithDetails | null,
    loading: false,
    error: null as string | null,
  }),

  actions: {
    async loadPersonnel() {
      this.loading = true
      this.error = null
      try {
        this.personnel = await adminService.getPersonnelList()
      } catch (error: any) {
        this.error = error.message || 'Failed to load personnel'
        throw error
      } finally {
        this.loading = false
      }
    },

    async loadPersonnelDetails(id: number) {
      this.loading = true
      this.error = null
      try {
        this.currentPersonnel = await adminService.getPersonnelDetails(id)
      } catch (error: any) {
        this.error = error.message || 'Failed to load personnel details'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createPersonnel(data: CreatePersonnelRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.createPersonnel(data)
        this.personnel.unshift(result)
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to create personnel'
        throw error
      } finally {
        this.loading = false
      }
    },

    async updatePersonnel(id: number, data: UpdatePersonnelRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.updatePersonnel(id, data)
        const index = this.personnel.findIndex((p) => p.id === id)
        if (index !== -1) {
          this.personnel[index] = result
        }
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to update personnel'
        throw error
      } finally {
        this.loading = false
      }
    },

    async assignSuperAdmin(personnelId: number) {
      this.loading = true
      this.error = null
      try {
        await adminService.assignSuperAdmin(personnelId)
        await this.loadPersonnelDetails(personnelId)
      } catch (error: any) {
        this.error = error.message || 'Failed to assign super admin'
        throw error
      } finally {
        this.loading = false
      }
    },

    async loadDepartments() {
      this.loading = true
      this.error = null
      try {
        this.departments = await adminService.getDepartmentList()
      } catch (error: any) {
        this.error = error.message || 'Failed to load departments'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createDepartment(data: CreateDepartmentRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.createDepartment(data)
        this.departments.unshift(result)
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to create department'
        throw error
      } finally {
        this.loading = false
      }
    },

    async updateDepartment(id: number, data: UpdateDepartmentRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.updateDepartment(id, data)
        const index = this.departments.findIndex((d) => d.id === id)
        if (index !== -1) {
          this.departments[index] = result
        }
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to update department'
        throw error
      } finally {
        this.loading = false
      }
    },

    async deleteDepartment(id: number) {
      this.loading = true
      this.error = null
      try {
        await adminService.deleteDepartment(id)
        this.departments = this.departments.filter((d) => d.id !== id)
      } catch (error: any) {
        this.error = error.message || 'Failed to delete department'
        throw error
      } finally {
        this.loading = false
      }
    },

    async loadMenus() {
      this.loading = true
      this.error = null
      try {
        this.menus = await adminService.getMenuList()
      } catch (error: any) {
        this.error = error.message || 'Failed to load menus'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createMenu(data: CreateMenuRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.createMenu(data)
        this.menus.unshift(result)
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to create menu'
        throw error
      } finally {
        this.loading = false
      }
    },

    async updateMenu(id: number, data: UpdateMenuRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.updateMenu(id, data)
        const index = this.menus.findIndex((m) => m.id === id)
        if (index !== -1) {
          this.menus[index] = result
        }
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to update menu'
        throw error
      } finally {
        this.loading = false
      }
    },

    async deleteMenu(id: number) {
      this.loading = true
      this.error = null
      try {
        await adminService.deleteMenu(id)
        this.menus = this.menus.filter((m) => m.id !== id)
      } catch (error: any) {
        this.error = error.message || 'Failed to delete menu'
        throw error
      } finally {
        this.loading = false
      }
    },

    async loadPermissions() {
      this.loading = true
      this.error = null
      try {
        this.permissions = await adminService.getPermissionList()
      } catch (error: any) {
        this.error = error.message || 'Failed to load permissions'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createPermission(data: CreatePermissionRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.createPermission(data)
        this.permissions.unshift(result)
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to create permission'
        throw error
      } finally {
        this.loading = false
      }
    },

    async deletePermission(id: number) {
      this.loading = true
      this.error = null
      try {
        await adminService.deletePermission(id)
        this.permissions = this.permissions.filter((p) => p.id !== id)
      } catch (error: any) {
        this.error = error.message || 'Failed to delete permission'
        throw error
      } finally {
        this.loading = false
      }
    },

    async loadRoles() {
      this.loading = true
      this.error = null
      try {
        this.roles = await adminService.getRoleList()
      } catch (error: any) {
        this.error = error.message || 'Failed to load roles'
        throw error
      } finally {
        this.loading = false
      }
    },

    async createRole(data: CreateRoleRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.createRole(data)
        this.roles.unshift(result)
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to create role'
        throw error
      } finally {
        this.loading = false
      }
    },

    async updateRole(id: number, data: UpdateRoleRequest) {
      this.loading = true
      this.error = null
      try {
        const result = await adminService.updateRole(id, data)
        const index = this.roles.findIndex((r) => r.id === id)
        if (index !== -1) {
          this.roles[index] = result
        }
        return result
      } catch (error: any) {
        this.error = error.message || 'Failed to update role'
        throw error
      } finally {
        this.loading = false
      }
    },

    async deleteRole(id: number) {
      this.loading = true
      this.error = null
      try {
        await adminService.deleteRole(id)
        this.roles = this.roles.filter((r) => r.id !== id)
      } catch (error: any) {
        this.error = error.message || 'Failed to delete role'
        throw error
      } finally {
        this.loading = false
      }
    },

    async assignPermissionsToRole(roleId: number, permissionIds: number[]) {
      this.loading = true
      this.error = null
      try {
        await adminService.assignPermissionsToRole(roleId, permissionIds)
      } catch (error: any) {
        this.error = error.message || 'Failed to assign permissions'
        throw error
      } finally {
        this.loading = false
      }
    },

    async assignDepartments(personnelId: number, departmentIds: number[]) {
      this.loading = true
      this.error = null
      try {
        await adminService.assignDepartments(personnelId, departmentIds)
        await this.loadPersonnelDetails(personnelId)
      } catch (error: any) {
        this.error = error.message || 'Failed to assign departments'
        throw error
      } finally {
        this.loading = false
      }
    },

    async assignRoles(personnelId: number, roleIds: number[]) {
      this.loading = true
      this.error = null
      try {
        await adminService.assignRoles(personnelId, roleIds)
        await this.loadPersonnelDetails(personnelId)
      } catch (error: any) {
        this.error = error.message || 'Failed to assign roles'
        throw error
      } finally {
        this.loading = false
      }
    },

    async initSuperAdmin() {
      this.loading = true
      this.error = null
      try {
        await adminService.initSuperAdmin()
      } catch (error: any) {
        this.error = error.message || 'Failed to init super admin'
        throw error
      } finally {
        this.loading = false
      }
    },

    async initMenus() {
      this.loading = true
      this.error = null
      try {
        await adminService.initMenus()
        await this.loadMenus()
      } catch (error: any) {
        this.error = error.message || 'Failed to init menus'
        throw error
      } finally {
        this.loading = false
      }
    },

    async initPermissions() {
      this.loading = true
      this.error = null
      try {
        await adminService.initPermissions()
        await this.loadPermissions()
      } catch (error: any) {
        this.error = error.message || 'Failed to init permissions'
        throw error
      } finally {
        this.loading = false
      }
    },

    async initSuperAdminAll() {
      this.loading = true
      this.error = null
      try {
        await adminService.initSuperAdminAll()
      } catch (error: any) {
        this.error = error.message || 'Failed to init super admin all'
        throw error
      } finally {
        this.loading = false
      }
    },
  },
})