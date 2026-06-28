<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { GitFork, Search as SearchIcon, Network } from 'lucide-vue-next'
import { VueFlow } from '@vue-flow/core'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import '@vue-flow/controls/dist/style.css'
import { graphApi } from '@/vec/api'
import type { Entity, Relation } from '@/vec/types/graph'

interface TreeNode {
  label: string
  id: string
  type: string
  children?: TreeNode[]
}

const TYPE_COLORS: Record<string, string> = {
  Task: '#409eff',
  Document: '#67c23a',
  Decision: '#e6a23c',
  Concept: '#909399',
  Organization: '#f56c6c',
  Product: '#9b59b6',
  Technology: '#00bcd4',
  Person: '#ff9800',
  Date: '#795548',
  URL: '#607d8b',
  Email: '#3f51b5',
  Amount: '#ff5722',
}

const activeTab = ref('tree')
const searchQuery = ref('')
const entities = ref<Entity[]>([])
const relations = ref<Relation[]>([])
const loading = ref(false)
const selectedEntityId = ref<string>('')

const filteredEntities = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return entities.value
  return entities.value.filter((e) => {
    const name = (e.name || '').toLowerCase()
    const type = (e.entity_type || '').toLowerCase()
    return name.includes(q) || type.includes(q)
  })
})

const treeData = computed<TreeNode[]>(() => {
  if (!filteredEntities.value.length) return []
  const groups = new Map<string, TreeNode[]>()

  for (const entity of filteredEntities.value) {
    const type = entity.entity_type || 'Other'
    if (!groups.has(type)) groups.set(type, [])
    const entityNode: TreeNode = {
      label: entity.name || `实体 #${entity.id}`,
      id: `entity-${entity.id}`,
      type: type,
      children: [],
    }
    const rels = relations.value.filter(
      (r) => r.source_entity_id === entity.id || r.target_entity_id === entity.id
    )
    for (const rel of rels) {
      const otherId =
        rel.source_entity_id === entity.id ? rel.target_entity_id : rel.source_entity_id
      const other = entities.value.find((e) => e.id === otherId)
      const isOutgoing = rel.source_entity_id === entity.id
      const arrow = isOutgoing ? '→' : '←'
      entityNode.children!.push({
        label: `${rel.relation_type || '关联'} ${arrow} ${other?.name || `#${otherId}`}`,
        id: `rel-${rel.id}`,
        type: 'relation',
      })
    }
    groups.get(type)!.push(entityNode)
  }

  return Array.from(groups.entries()).map(([type, children]) => ({
    label: `${type} (${children.length})`,
    id: `type-${type}`,
    type: '__group__',
    children,
  }))
})

const graphNodes = computed(() => {
  const entities = filteredEntities.value
  const count = entities.length
  if (count === 0) return []

  // 使用圆形布局计算初始位置
  const radius = Math.max(150, count * 60)
  const centerX = 400
  const centerY = 250

  return entities.map((e, index) => {
    const angle = (2 * Math.PI * index) / count
    const x = centerX + radius * Math.cos(angle)
    const y = centerY + radius * Math.sin(angle)

    return {
      id: String(e.id),
      type: 'default',
      label: `${e.name || `#${e.id}`}\n${e.entity_type || ''}`,
      class: `kg-node kg-node-${(e.entity_type || 'default').toLowerCase()}`,
      position: { x, y },
      style: {
        borderLeftWidth: '3px',
        borderLeftStyle: 'solid',
        borderLeftColor: TYPE_COLORS[e.entity_type || ''] || '#409eff',
        borderRadius: '8px',
        fontSize: '12px',
        padding: '8px 12px',
        minWidth: '80px',
        textAlign: 'center' as const,
        background: '#fff',
        border: '1px solid #dcdfe6',
      },
    }
  })
})

const graphEdges = computed(() => {
  const ids = new Set(filteredEntities.value.map((e) => String(e.id)))
  return relations.value
    .filter((r) => ids.has(String(r.source_entity_id)) && ids.has(String(r.target_entity_id)))
    .map((r) => ({
      id: `e-${r.id}`,
      source: String(r.source_entity_id),
      target: String(r.target_entity_id),
      label: r.relation_type || '',
      animated: false,
      style: { stroke: '#c0c4cc', strokeWidth: 1.5 },
      labelStyle: { fill: '#606266', fontWeight: 400, fontSize: 11 },
      labelBgStyle: { fill: '#f5f7fa', fillOpacity: 0.9 },
      labelBgPadding: [4, 2] as [number, number],
      labelBgBorderRadius: 3,
      markerEnd: 'arrow-closed',
    }))
})

const selectedEntity = computed(() => {
  if (!selectedEntityId.value) return null
  return entities.value.find((e) => String(e.id) === selectedEntityId.value) || null
})

const selectedEntityRelations = computed(() => {
  if (!selectedEntity.value) return []
  const id = selectedEntity.value.id
  return relations.value.filter(
    (r) => r.source_entity_id === id || r.target_entity_id === id
  )
})

const loadData = async () => {
  loading.value = true
  try {
    entities.value = await graphApi.data()
    const allRels: Relation[] = []
    await Promise.all(
      entities.value.map(async (entity) => {
        try {
          const rels = await graphApi.entityRelations(String(entity.id))
          allRels.push(...rels)
        } catch {
          // ignore per-entity errors
        }
      })
    )
    const seen = new Set<number>()
    const unique: Relation[] = []
    for (const r of allRels) {
      if (!seen.has(r.id)) {
        seen.add(r.id)
        unique.push(r)
      }
    }
    relations.value = unique
  } finally {
    loading.value = false
  }
}

const flowKey = ref(0)

const onNodeClick = (event: any) => {
  selectedEntityId.value = event.node?.id || ''
}

const onFlowInit = () => {
  // VueFlow 初始化完成
}

watch(activeTab, async (newTab) => {
  if (newTab === 'graph') {
    flowKey.value++
    await nextTick()
  }
})

onMounted(loadData)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">知识图谱</h1>
        <p class="page-subtitle">
          {{ filteredEntities.length }} 个实体，{{ relations.length }} 条关系
        </p>
      </div>
    </div>

    <el-card class="mb-6">
      <el-input v-model="searchQuery" placeholder="搜索实体名称或类型">
        <template #prefix>
          <SearchIcon class="w-4 h-4 text-gray-400" />
        </template>
      </el-input>
    </el-card>

    <el-card v-loading="loading">
      <div class="tab-header">
        <div
          class="tab-item"
          :class="{ active: activeTab === 'tree' }"
          @click="activeTab = 'tree'"
        >
          <GitFork class="w-4 h-4" />
          <span>树形结构</span>
        </div>
        <div
          class="tab-item"
          :class="{ active: activeTab === 'graph' }"
          @click="activeTab = 'graph'"
        >
          <Network class="w-4 h-4" />
          <span>关系图</span>
        </div>
      </div>

      <!-- 树形视图 -->
      <div v-if="activeTab === 'tree'">
        <el-tree
          v-if="treeData.length"
          :data="treeData"
          node-key="id"
          default-expand-all
          :props="{ label: 'label', children: 'children' }"
        >
          <template #default="{ node, data }">
            <span class="tree-node">
              <el-tag
                v-if="data.type === '__group__'"
                color="#f0f2f5"
                size="small"
                :style="{ borderColor: '#c0c4cc', color: '#303133' }"
              >
                {{ data.label }}
              </el-tag>
              <template v-else-if="data.type === 'relation'">
                <el-tag type="success" size="small">关系</el-tag>
                <span class="text-gray-600">{{ node.label }}</span>
              </template>
              <template v-else>
                <el-tag type="primary" size="small">{{ data.type }}</el-tag>
                <span>{{ node.label }}</span>
              </template>
            </span>
          </template>
        </el-tree>
        <el-empty v-else description="暂无实体数据" />
      </div>

      <!-- 关系图视图 -->
      <div v-if="activeTab === 'graph'">
        <div class="graph-wrapper">
          <VueFlow
            v-if="graphNodes.length"
            :key="flowKey"
            :nodes="graphNodes"
            :edges="graphEdges"
            :fit-view="true"
            :pan-on-drag="true"
            :zoom-on-scroll="true"
            :nodes-draggable="true"
            class="kg-flow"
            @node-click="onNodeClick"
            @init="onFlowInit"
          >
            <Background />
            <Controls />
          </VueFlow>
          <el-empty v-else description="暂无数据，请先提取实体" />
        </div>
        <div v-if="selectedEntity" class="entity-detail">
          <div class="detail-header">
            <el-tag size="small">{{ selectedEntity.entity_type }}</el-tag>
            <span class="detail-name">{{ selectedEntity.name }}</span>
          </div>
          <p v-if="selectedEntity.description" class="detail-desc">
            {{ selectedEntity.description }}
          </p>
          <div v-if="selectedEntityRelations.length" class="detail-rels">
            <div class="detail-rels-title">关联关系：</div>
            <div
              v-for="rel in selectedEntityRelations"
              :key="rel.id"
              class="detail-rel-item"
            >
              <el-tag type="success" size="small">{{ rel.relation_type }}</el-tag>
              <span>
                {{ rel.source_entity_id === selectedEntity.id ? '→' : '←' }}
                {{ entities.find((e) => e.id === (rel.source_entity_id === selectedEntity.id ? rel.target_entity_id : rel.source_entity_id))?.name || '未知' }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<style>
.tree-node {
  display: flex;
  align-items: center;
  gap: 8px;
}

.graph-wrapper {
  height: 500px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  overflow: hidden;
  background: #fafafa;
}

.kg-flow {
  width: 100%;
  height: 100%;
}

.kg-flow .vue-flow__edge-textwrapper .vue-flow__edge-textbg {
  fill: #f5f7fa;
  fill-opacity: 0.9;
}

.kg-flow .vue-flow__edge-textwrapper .vue-flow__edge-text {
  fill: #606266;
  font-size: 11px;
}

.entity-detail {
  margin-top: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.detail-name {
  font-weight: 600;
  font-size: 16px;
}

.detail-desc {
  color: #606266;
  font-size: 14px;
  margin-bottom: 8px;
}

.detail-rels {
  margin-top: 12px;
}

.detail-rels-title {
  font-weight: 500;
  margin-bottom: 8px;
  color: #303133;
}

.detail-rel-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}
</style>
