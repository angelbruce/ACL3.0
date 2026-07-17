@echo off
REM ACL权限管理系统测试脚本 (Windows)

set BASE_URL=http://localhost:3007

echo ==========================================
echo ACL权限管理系统 - API测试脚本
echo ==========================================
echo.

REM 1. 初始化超级管理员角色
echo 1. 初始化超级管理员角色...
curl -X POST "%BASE_URL%/init-super-admin"
echo.
echo.

REM 2. 初始化菜单
echo 2. 初始化菜单...
curl -X POST "%BASE_URL%/init-menus"
echo.
echo.

REM 3. 初始化权限
echo 3. 初始化权限...
curl -X POST "%BASE_URL%/init-permissions"
echo.
echo.

REM 4. 分配所有权限给超级管理员
echo 4. 分配所有权限给超级管理员...
curl -X POST "%BASE_URL%/init-super-admin-all"
echo.
echo.

REM 5. 获取所有角色
echo 5. 获取所有角色...
curl -X GET "%BASE_URL%/roles"
echo.
echo.

REM 6. 获取所有菜单
echo 6. 获取所有菜单...
curl -X GET "%BASE_URL%/menus"
echo.
echo.

REM 7. 获取所有权限
echo 7. 获取所有权限...
curl -X GET "%BASE_URL%/permissions"
echo.
echo.

REM 8. 创建部门
echo 8. 创建部门...
curl -X POST "%BASE_URL%/departments" ^
  -H "Content-Type: application/json" ^
  -d "{\"name\": \"技术部\", \"description\": \"负责技术研发\"}"
echo.
echo.

REM 9. 创建人员（示例）
echo 9. 创建人员...
curl -X POST "%BASE_URL%/personnel" ^
  -H "Content-Type: application/json" ^
  -d "{\"name\": \"测试用户\", \"email\": \"test@example.com\"}"
echo.
echo.

REM 10. 获取所有人员
echo 10. 获取所有人员...
curl -X GET "%BASE_URL%/personnel"
echo.
echo.

REM 11. 获取所有部门
echo 11. 获取所有部门...
curl -X GET "%BASE_URL%/departments"
echo.
echo.

echo ==========================================
echo 测试完成！
echo ==========================================
pause
