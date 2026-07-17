#!/bin/bash

# ACL权限管理系统测试脚本

BASE_URL="http://localhost:3007"

echo "=========================================="
echo "ACL权限管理系统 - API测试脚本"
echo "=========================================="
echo ""

# 1. 初始化超级管理员角色
echo "1. 初始化超级管理员角色..."
curl -s -X POST "$BASE_URL/init-super-admin" | jq '.'
echo ""

# 2. 初始化菜单
echo "2. 初始化菜单..."
curl -s -X POST "$BASE_URL/init-menus" | jq '.'
echo ""

# 3. 初始化权限
echo "3. 初始化权限..."
curl -s -X POST "$BASE_URL/init-permissions" | jq '.'
echo ""

# 4. 分配所有权限给超级管理员
echo "4. 分配所有权限给超级管理员..."
curl -s -X POST "$BASE_URL/init-super-admin-all" | jq '.'
echo ""

# 5. 获取所有角色
echo "5. 获取所有角色..."
curl -s -X GET "$BASE_URL/roles" | jq '.'
echo ""

# 6. 获取所有菜单
echo "6. 获取所有菜单..."
curl -s -X GET "$BASE_URL/menus" | jq '.'
echo ""

# 7. 获取所有权限
echo "7. 获取所有权限..."
curl -s -X GET "$BASE_URL/permissions" | jq '.'
echo ""

# 8. 创建部门
echo "8. 创建部门..."
DEPT_RESPONSE=$(curl -s -X POST "$BASE_URL/departments" \
  -H "Content-Type: application/json" \
  -d '{"name": "技术部", "description": "负责技术研发"}')
echo "$DEPT_RESPONSE" | jq '.'
DEPT_ID=$(echo "$DEPT_RESPONSE" | jq -r '.id')
echo "创建的部门ID: $DEPT_ID"
echo ""

# 9. 创建人员（示例）
echo "9. 创建人员..."
PERSONNEL_RESPONSE=$(curl -s -X POST "$BASE_URL/personnel" \
  -H "Content-Type: application/json" \
  -d '{"name": "测试用户", "email": "test@example.com"}')
echo "$PERSONNEL_RESPONSE" | jq '.'
PERSONNEL_ID=$(echo "$PERSONNEL_RESPONSE" | jq -r '.id')
echo "创建的人员ID: $PERSONNEL_ID"
echo ""

# 10. 分配超级管理员角色给人员
echo "10. 分配超级管理员角色给人员..."
curl -s -X POST "$BASE_URL/personnel/$PERSONNEL_ID/assign-super-admin" | jq '.'
echo ""

# 11. 获取人员完整信息
echo "11. 获取人员完整信息..."
curl -s -X GET "$BASE_URL/personnel/$PERSONNEL_ID/details" | jq '.'
echo ""

# 12. 分配部门给人员
echo "12. 分配部门给人员..."
curl -s -X POST "$BASE_URL/personnel/$PERSONNEL_ID/assign-departments" \
  -H "Content-Type: application/json" \
  -d "{\"personnel_id\": $PERSONNEL_ID, \"department_ids\": [$DEPT_ID]}" | jq '.'
echo ""

# 13. 获取人员所属部门
echo "13. 获取人员所属部门..."
curl -s -X GET "$BASE_URL/personnel/$PERSONNEL_ID/departments" | jq '.'
echo ""

# 14. 获取人员角色
echo "14. 获取人员角色..."
curl -s -X GET "$BASE_URL/personnel/$PERSONNEL_ID/roles" | jq '.'
echo ""

# 15. 检查是否为超级管理员
echo "15. 检查是否为超级管理员..."
curl -s -X GET "$BASE_URL/personnel/$PERSONNEL_ID/is-super-admin" | jq '.'
echo ""

# 16. 获取所有人员
echo "16. 获取所有人员..."
curl -s -X GET "$BASE_URL/personnel" | jq '.'
echo ""

# 17. 获取所有部门
echo "17. 获取所有部门..."
curl -s -X GET "$BASE_URL/departments" | jq '.'
echo ""

echo "=========================================="
echo "测试完成！"
echo "=========================================="
