<template>
  <div class="flex h-screen bg-white">
    <div class="w-16 bg-gray-100 py-12">
      <div class="flex flex-col items-center py-2 gap-3" v-for="item in nodeTypes" :key="item.type" >
        <button @click="addNode(item.type)"
          class="w-10 h-10 rounded-lg flex items-center justify-center text-white cursor-pointer transition-all hover:scale-105"
          :class="getButtonClass(item.type)" :title="item.label">
          <component :is="getNodeIcon(item.type)" class="w-5 h-5" />
        </button>
        <div class="text-xs text-gray-500">{{ item.label }}</div>
      </div>
    </div>

    <div class="flex flex-1  flex-col"  id="ddd">
        <div class="h-14 bg-gray-50 border-b flex items-center justify-between px-4">
          <input v-model="flowName" type="text" placeholder="请输入工作流名称"
            class="flex-1 max-w-md px-3 py-2 border border-gray-200 rounded-lg focus:outline-none focus:border-blue-500" />
          <div class="flex gap-2">
            <button @click="doConnect"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors">
              连接
            </button>
            <button @click="deleteSelected"
              class="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors">
              删除
            </button>
            <button @click="router.push('/flows')"
              class="px-4 py-2 bg-white text-black rounded-lg border border-gray-300 hover:bg-white-500 transition-colors">
              取消
            </button>
            <button @click="handleSubmit"
              class="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors">
              保存
            </button>
          </div>
      </div>

      <div class="flex-1 flex flex-row" >
        <!-- 节点容器 -->
        <div ref="containerRef" class="flex flex-1"></div>
        <!-- 属性容器 -->
        <div class="flex flex-col w-1/6 border border-gray-200 rounded-sm p-4" v-show="current.type === 'agent'">
          <div class="flex w-full flex-row items-start justify-start">
            <div class="flex w-full flex-col items-start justify-start gap-2 w-full ">
              <div class="flex items-center justify-start align-left text-lg font-bold text-gray-800">
                <b><label id="node" v-bind="current.value"></label></b>
              </div>
                <div class="flex items-center text-sm align-left text-gray-800">Agent</div>
                <select class="flex w-full px-2 py-1 border border-gray-300  text-sm rounded-md" v-model="current.agent">
                  <option v-for="agent in agents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
                </select>
              
                <div class="flex items-center text-sm align-left text-gray-800">角色定义</div>
                <textarea class="flex align-left w-full px-2 py-1 border border-gray-300 rounded-md " v-model="current.prompt" style="min-height:100px;"></textarea>

                <div class="flex w-full align-left items-center text-sm align-left text-gray-800">完成路径</div>
                <div class="flex w-full  align-left items-center text-sm align-left text-gray-800 border-b  border-gray-100" v-for="path in current.fromPaths" :key="path.id">
                  <input class="mx-2" type="checkbox" :id="path.id" :name="path.id" :value="path.id" v-model="path.checked">
                  <label for="path.id">{{ path.src.value }} -> {{ path.value }}</label>
                </div>

                <div class="flex w-full align-left items-center text-sm align-left text-gray-800">完成度</div>
                <select class="flex w-full px-2 py-1 border border-gray-300 rounded-md text-sm " v-model="current.degree">
                    <option value="100">所有完成</option>
                    <option value="1">任何一项</option>
                </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { loadmodule } from '@/utils/mx';

onMounted(async () => {
  await loadmodule();
});

import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router'
import { Play, Circle, GitBranch, Square, CircleDot, ArrowBigUp } from 'lucide-vue-next';
import { useFlowStore,useAgentStore } from '@/stores';
import { PathStore, NodeInfo, Edge, FlowData, PathInfo, Style, NodeInfoEx ,nodeTypes} from '@/views/flows/PathStore';
const route = useRoute()
var flowStore = useFlowStore();
const router = useRouter()
const flowId = Number(route.params.id)
const pathStore = new PathStore();
const agentStore = useAgentStore();

var current = ref<NodeInfoEx>({
  id:'-1',
  type:'',
  value:'current',
  prompt:'',
  agent:null,
  degree:'100',
  paths:[],
  x:0,
  y:0,
  fromPaths:[],
})
const props = defineProps<{
  flowData?: FlowData;
}>();

const emit = defineEmits<{
  (e: 'save', data: FlowData): void;
}>();
const flowName = ref('');
const containerRef = ref<HTMLElement | null>(null);
let nodeIdCounter = 1;

const NODE_W = 160;
const NODE_H = 60;
let lastX = 200;
let lastY = 200;
let graph: any = null;
const nextNodeId = () => {
  return `node_${Date.now()}_${nodeIdCounter++}`;
};
const getButtonClass = (type: string) => {
  const classes: Record<string, string> = {
    start: 'bg-green-500',
    agent: 'bg-yellow-500',
    terminate: 'bg-blue-500',
    end: 'bg-red-500',
  };
  return classes[type] || 'bg-gray-500';
};
const getNodeIcon = (type: string) => {
  const icons: Record<string, any> = {
    start: Play,
    agent: Square,
    terminate: CircleDot,
    end: Circle,
  };
  return icons[type] || Square;
};
const addNode = (type: string) => {
  if (!graph)
    return;
  const label = type === 'start' ? '开始' : type === 'end' ? '结束' : type === 'terminate' ? '终止' : '动作';
  const w = type === 'start' || type === 'end' ? 40 : NODE_W;
  const h = type === 'start' || type === 'end' ? 40 : NODE_H;
  const id = nextNodeId();
  const style = getNodeStyle(type);
  const parent = graph.getDefaultParent();
  var cell = graph.insertVertex(parent, id, label, lastX, lastY, w, h, style);
  pathStore.refresh(cell);
  lastX = lastX + w + 50;
  if (lastX > 700) {
    lastX = 200;
    lastY += h + 50;
  }
};
const getNodeStyle = (type: string): string => {
  const styles: Record<string, string> = {
    start: 'shape=ellipse;fillColor=#ffddaa;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;type=start;',
    agent: 'fillColor=#eac133;strokeColor=#6666aa;lineWidth=2;rounded=2;fontColor=#000000;type=agent;',
    terminate: 'shape=ellipse;fillColor=#eedd33;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;type=terminate;',
    end: 'shape=ellipse;fillColor=#112233;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#ffffff;type=end;',
  };
  return styles[type] || styles.agent;
};
const getCellNodeType = (cell: any): string => {
  if (!cell)
    return '';
  const style = cell.style || '';
  const match = style.match(/type=([^;]*)/);
  return match ? match[1] : '';
};
const doConnect = () => {
  if (!graph)
    return;
  const selections = graph.getSelectionCells();
  if (!selections || selections.length < 2)
    return;
  const parent = graph.getDefaultParent();
  let last: any = null;
  for (let i = 0; i < selections.length; i++) {
    const node = selections[i];
    if (!node.vertex)
      continue;
    if (last == null) {
      last = node;
      continue;
    }
    const ltype = getCellNodeType(last);
    const ntype = getCellNodeType(node);
    if (ltype !== '' && ntype !== '' && ltype !== 'agent' && ntype !== 'agent') {
      if (ltype === ntype) {
        last = node;
        continue;
      }
    }
    let from = last;
    let to = node;
    if ((ltype === 'agent' && (ntype !== 'agent' && ntype !== 'terminate' && ntype !== 'end')) ||
      (ltype !== 'start' && ntype === 'start') ||
      (ltype === 'end' && ntype !== 'end')) {
      from = node;
      to = last;
    }

    let value = '下一步';
    if (ltype === 'agent' || ntype === 'agent') value = '下一步';
    const lineStyle = 'strokeWidth=1;strokeColor=#6b7280;endArrow=classic;endFill=true;';
    graph.insertEdge(parent, null, value, from, to, lineStyle);
    last = node;
  }

  for(const cell in selections) {
    if(cell.vertex) {
      pathStore.refresh(cell);
    }
  }
};
const deleteSelected = () => {
  if (!graph)
    return;
  const selections = graph.getSelectionCells();
  if (!selections || selections.length === 0)
    return;
  for (const cell of selections) {
    const id = cell.getId();
    if (id === 'start')
      continue;
  }
  graph.removeCells(selections);
};

var nodeStart: any|null  = null


const onShowProps = ()=> {
  var selection = graph.getSelectionCells();
  var empty = {id: '', value: '', prompt: '', agent: null, degree: '100', paths: [], type: '',fromPaths:[]};
    if(selection.length === 0 || !selection[0].vertex) {
      console.log(1);
      current.value = empty;
      return;
    }

    if(current.value &&  current.value.id && current.value.id === selection[0].id) {
      console.log(2);
      return;
    }

    var sel = selection[0];
    var nt = getCellNodeType(sel);
    if(nt !== 'agent'){
      console.log(3);
      current.value = empty;
      return;
    } 

    pathStore.refresh(sel);
    var node = pathStore.getNodeInfo(sel.id);
    if(!node){
      current.value = empty;
      return;
    } 

    console.log('--node-- ', node)

    current.value = node;
    return false;
}


const initGraph = () => {
  if (!containerRef.value)
    return;
  const mxGraph = (window as any).mxGraph;
  const mxRubberband = (window as any).mxRubberband;
  const mxKeyHandler = (window as any).mxKeyHandler;
  const mxConnectionHandler = (window as any).mxConnectionHandler;
  graph = new mxGraph(containerRef.value);
  graph.setCellsSelectable(true);
  graph.setCellsMovable(true);
  graph.setMultigraph(false);
  graph.isAllowDanglingEdges(false);
  graph.addListener('click', ()=> { onShowProps();});

  graph.model.valueForCellChanged = function(cell, value) {
    cell.value = value;
    if(cell) {
      if(cell.vertex) {
        pathStore.refresh(cell);
      } else if(cell.edge) {
        pathStore.refresh(cell.target);
        pathStore.refresh(cell.source);
      }
    }
    return value;
  }

  new mxRubberband(graph);
  const connHandler = new mxConnectionHandler(graph);
  connHandler.setEnabled(false);
  new mxKeyHandler(graph);
  const parent = graph.getDefaultParent();
  const startStyle = 'shape=ellipse;fillColor=#ffddaa;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;type=start;';
  nodeStart = graph.insertVertex(parent, 'start', '开始', 50, 200, 60, 60, startStyle);
  
  if (props.flowData) {
    loadFlow(props.flowData);
  }
};
const loadFlow = (data: FlowData) => {
  if (!graph) return;
  if(!data) return;

  console.log(data)
  
  if(nodeStart) graph.removeCells([nodeStart]);

  flowName.value = data.name;
  const parent = graph.getDefaultParent();
  graph.model.beginUpdate();
  let verMap:Record<string, any> = {};
  data.vertices.forEach((v) => {
    const style = getNodeStyle(v.type);
    const w = v.type === 'start' || v.type === 'end' ? 60 : NODE_W;
    const h = v.type === 'start' || v.type === 'end' ? 60 : NODE_H;
    var cell = graph.insertVertex(parent, v.id, v.value || '', v.x || 150, v.y || 120, w, h, style);
    verMap[cell.id] = cell;
  });

  graph.model.endUpdate();
  graph.view.refresh();
  graph.view.invalidate();
  
  data.edges.forEach((e) => {
    const srcCell = verMap[e.src];
    const tgtCell = verMap[e.target];
    if (srcCell && tgtCell) {
      graph.insertEdge(parent, e.id, e.value || '', srcCell, tgtCell, e.style || '');
    }
  });

  pathStore.load(data);
};

const handleSubmit = async () => {
  if (!flowName.value.trim()) {
    alert('请输入工作流名称');
    return;
  }

  const data = exportFlow();

  if(flowId ) {
    flowStore.updateFlow(flowId, data);
  } else {
    flowStore.createFlow(data);
  }
  
  emit('save', data);
  router.push('/flows')
};

const exportFlow = (): FlowData => {
  if (!graph) {
    return { name: flowName.value, vertices: [], edges: [] };
  }
  const model = graph.getModel();
  const parent = graph.getDefaultParent();
  const vertices: NodeInfo[] = [];
  const edges: Edge[] = [];
  const cells = model.cells;
  for (const key in cells) {
    const cell = cells[key];
    if (!cell) continue;

    pathStore.refresh(cell);
    if (cell.vertex && cell.getParent() === parent) {
      const geometry = cell.getGeometry();
      const id = cell.getId();
      var node = pathStore.getNodeInfo(id);
      if(!node) continue;

      vertices.push({
        id:id,
        value: cell.value || '',
        type:  node.type,
        prompt: node.prompt || '',
        agent: node.agent,
        degree: node.degree || null,
        paths: node.paths || [],
        x: geometry?.x || 0,
        y: geometry?.y || 0,
      });
    }
    else if (cell.isEdge() && cell.getParent() === parent) {
      const src = cell.getTerminal(true);
      const tgt = cell.getTerminal(false);
      if (src && tgt) {
        edges.push({
          id: cell.getId(),
          src: src.getId(),
          target: tgt.getId(),
          value: cell.value || '',
          style: cell.style || '',
        });
      }
    }
  }
  return {
    name: flowName.value,
    config : {
      vertices,
      edges,
    },
  };
};

const waitForMxGraph = (callback: () => void) => {
  if ((window as any).mxGraph) {
    callback();
  } else { 
    setTimeout(() => waitForMxGraph(callback), 50);
 }
};

const agents = ref([]);
const loadAgentData = ()=> {
  agentStore.fetchAgents().then((data)=>{
    agents.value = agentStore.agents;
  });
}

const loadFlowData = ()=> {
  loadAgentData();
  if(flowId) {
    flowStore.fetchFlow(flowId).then((data)=>{
      let flowData = new ref<FlowData>({});
      flowData.name = data.name;
      flowData.vertices =data.config.vertices;
      flowData.edges=data.config.edges;
      loadFlow(flowData);
    });
  }
};

onMounted(() => {
 waitForMxGraph(initGraph);
 loadFlowData();
});


onUnmounted(() => {
 graph = null;
});



</script>