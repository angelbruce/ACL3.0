export interface NodeInfo {
    id: string;
    value: string;
    prompt: string;
    type: string;
    agent: string | null;
    degree: number | 1;
    paths: string[];
    x?: number;
    y?: number;
    w?: number;
    h?: number;
}
export interface Edge {
    id: string;
    src: string;
    target: string;
    value: string;
    style: string;
}

export interface FlowData {
    name: string;
    vertices: NodeInfo[];
    edges: Edge[];
}

export interface PathInfo {
    id: string;
    src: any;
    target: any;
    value: string;
    checked: boolean;
}

export const nodeTypes = [
    { type: 'start', label: '开始' },
    { type: 'agent', label: '动作' },
    { type: 'input', label: '输入' },
    // { type: 'output', label: '输出' },
    // { type: 'terminate', label: '终止' },
    { type: 'end', label: '结束' },
];

export interface NodeInfoEx extends NodeInfo {
  fromPaths: PathInfo[];
}

export class PathStore {
    private nodeMap: Record<string, NodeInfoEx> = {};
    constructor() {
        this.nodeMap = {};
    }

    public load(data: FlowData) {
        if (!data || !data.edges || !data.vertices) return;
        
        var vertices = data.vertices || [];
        for (let vertex of vertices) {
            this.nodeMap[vertex.id] = {
                id: vertex.id || '',
                value: vertex.value || '',
                prompt: vertex.prompt || '',
                type: vertex.type || '',
                agent: vertex.agent || null,
                degree: vertex.degree || 1,
                paths: vertex.paths || [],
                fromPaths:  [],
            };
        }

        var pathEdges: Record<string, PathInfo> = {};
        var edges = data.edges || [];
        for (let edge of edges) {
            var src = edge.src || '';
            var target = edge.target || '';
            pathEdges[edge.id] = {
                id: edge.id,
                src: { id: edge.src || '', value: this.nodeMap[src].value || '' },
                target: { id: edge.target || '', value: this.nodeMap[target].value || '' },
                value: edge.value || '',
                checked: false
            };
        }

        for (let vertex of vertices) {
            var paths = vertex.paths || [];
            for (let path of paths) {
                var edge = pathEdges[path];
                if (edge) {
                    var clonedEdge = { ...edge };
                    clonedEdge.checked = true;
                    this.nodeMap[vertex.id].fromPaths.push(clonedEdge);
                }
            }
        }
    }

    public refresh(cell: any) {
        if (!cell) return;
        if (!cell.vertex) return

        var style = new Style(cell);
        var nodeInfo = this.nodeMap[cell.id];
        if (!nodeInfo) {
            nodeInfo = {
                id: cell.id || '',
                value: cell.value || '',
                prompt: '',
                type: style.type(),
                agent: style.agent() || null,
                degree: 1,
                paths: [] as string[],
                fromPaths:  [],
            };
            this.nodeMap[cell.id] = nodeInfo;
        }

        var existsEdges = nodeInfo.fromPaths.map((e) => e.id);

        var edges = cell.edges || [];
        var newEdgeMap: Record<string, Edge> = {};
        for (let edge of edges) {
            if (edge.target.id != cell.id) continue;

            var id = edge.id;
            newEdgeMap[id] = edge;
            var src = edge.source || {id:'',value:''};
            var target = edge.target || {id:'',value:''};
            if (!existsEdges.includes(id)) {
                var data  = {
                    id: id || '',
                    src: { id: src.id || '', value: src.value || '' },
                    target: { id: target.id || '', value: target.value || '' },
                    value: edge.value || '',
                    checked: false,
                };
                console.log('---push data to from paths---', data);
                this.nodeMap[cell.id].fromPaths.push(data);
            }
        }

        for (let id in existsEdges) {
            if (!newEdgeMap.hasOwnProperty(id)) {
                for(let i = 0; i < nodeInfo.fromPaths.length; i++){
                    if(nodeInfo.fromPaths[i].id === id){
                        nodeInfo.fromPaths.splice(i, 1);
                        break;
                    }
                }
            }
        }

        var paths = nodeInfo.fromPaths || [];
        nodeInfo.paths = paths.filter(e=>e.checked).map((e) => e.id);
    }

    public getCellPaths(cell: any) : string[] {
        var nodeInfo = this.nodeMap[cell.id];
        if (!nodeInfo) return [];

        var paths = nodeInfo.fromPaths || [];
        return paths.filter(e=>e.checked).map((e) => e.id);
    }

    public getNodeInfo(id: string) {
        
        var nodeInfo = this.nodeMap[id] || null;
        if (!nodeInfo) return null;
        var fromPaths = nodeInfo.fromPaths || [];
        var paths = fromPaths.filter(e=>e.checked).map((e) => e.id);
        nodeInfo.paths = paths;
        return nodeInfo;
    }
}


export class Style {
    private dc: Record<string, any> = {}

    constructor(cell: any) {
        var str = cell.style || '';
        this.init(str);
    }


    public type(val?: string) {
        if (val) {
            this.set('type', val);
        }
        return this.get('type');
    }

    public agent(val?: string) {
        if (val) {
            this.set('agent', val);
        }
        return this.get('agent');
    }

    public toString() {
        var str = '';
        for (var k in this.dc) {
            str += k + '=' + this.dc[k] + ';'
        }

        return str;
    }

    public get(k: string) {
        if (!k) return null;
        if (!this.dc.hasOwnProperty(k)) return null;
        return this.dc[k];
    }

    public set(k: string, v: string) {
        if (!k) return;
        this.dc[k] = v;
    }

    public remove(k: string) {
        if (!k) return null;

        if (!this.dc.hasOwnProperty(k)) return null;
        delete this.dc[k];
    }

    public init(str: string) {
        var segs = str.split(';');
        for (var i = 0; i < segs.length; i++) {
            var seg = segs[i]
            var kv = seg.split('=');
            var k = kv[0];
            var v = kv[1];
            this.dc[k] = v;
        }
    }

}
