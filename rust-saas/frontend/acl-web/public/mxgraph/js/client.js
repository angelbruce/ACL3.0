var graph = null;
var parent = null;

function onInit() {
    initialize();
    loadAgents();
    if (flowId) loadConfig();
}

function query(name) {
    var url = window.location.href;
    var idx = url.indexOf('?')
    if (idx == -1) return null;
    var u = url.substring(idx + 1);
    var array = u.split('&')
    for (var i = 0; i < array.length; i++) {
        var seg = array[i].split('=');
        if (seg[0] == name) {
            if (seg.length == 1) return null;
            else return seg[1];
        }
    }
}

var flowId = null;
function initialize() {
    flowId = query('flowId');
    document.getElementById('flowId').value = flowId;
    graph = new mxGraph(document.getElementById('c'), null, null, null);
    parent = graph.getDefaultParent();
    document.getElementById('c').addEventListener('click', putMod)
}

function loadAgents() {
    showMods();
    fetch('http://local.res/agents').then(resp => {
        console.log(resp);
        var json = resp.json();
        json.then((data) => {
            console.log(data);
            if (data.Success) {
                //var models = document.getElementById('models');
                var datas = data.Data || [];
                //datas.push({ Id: -100, Name: '提示词', type: 'prompt' });
                //datas.push({ Id: -101, Name: '动作', type: 'ask' });
                //datas.push({ Id: -102, Name: '结束', type: 'over' });
                //datas.push({ Id: -103, Name: '终止', type: 'terminate' });
                //datas.push({ Id: -105, Name: '执行', type: 'terminate' });
                var ele = document.getElementById('agent');
                for (var i = 0; i < data.Data.length; i++) {
                    var d = data.Data[i];
                    //var div = document.createElement('div');
                    //div.innerText = d.Name;
                    //console.log(d.Name);
                    //div.className = 'model-item';
                    //div.setAttribute('data-id', d.Id + '');
                    //div.setAttribute('data-type', d.type ? d.type : 'agent');
                    //models.appendChild(div);
                    var opt = document.createElement('option');
                    opt.value = d.Id;
                    opt.innerText = d.Name;
                    ele.appendChild(opt);
                }

                //addUseListener();

            } else {
                alert(d.Message);
            }
        });

    }).catch(e => {
        console.log(e);
    })
}


function showMods() {
    var models = document.getElementById('models');
    var datas = [];
    datas.push({ Id: -100, Name: '提示词', type: 'prompt' });
    datas.push({ Id: -101, Name: '动作', type: 'ask' });
    datas.push({ Id: -102, Name: '结束', type: 'over' });
    datas.push({ Id: -103, Name: '终止', type: 'terminate' });
    datas.push({ Id: -105, Name: '执行', type: 'terminate' });
    for (var i = 0; i < datas.length; i++) {
        var d = datas[i];
        var div = document.createElement('div');
        div.innerText = d.Name;
        div.className = 'model-item';
        div.setAttribute('data-id', d.Id + '');
        div.setAttribute('data-type', d.type ? d.type : 'agent');
        models.appendChild(div);
    }

    addUseListener();
}

function addUseListener() {
    var items = document.querySelectorAll('.model-item');
    for (var i = 0; i < items.length; i++) {
        var item = items[i];
        item.addEventListener('click', useModel);
    }
}

function removeUseListener() {
    var items = document.querySelectorAll('.model-item');
    for (var i = 0; i < items.length; i++) {
        var item = items[i];
        item.removeEventListener('click', useModel);
    }
}

var current = { Id: 0, Name: '', Definition: '', type: 'agent' };
function useModel(e) {
    console.log(e);
    var node = e.target;
    var id = node.getAttribute('data-id');
    var type = node.getAttribute('data-type');
    var name = node.innerHTML;
    if (id == null) return;
    var x = e.x; var y = e.y;
    current.Id = id;
    current.Name = name;
    current.type = type;
    node.style.cursor = 'cross';
    return false;
}

var lineStyle = ''

function Style(st, type) {
    this.dc = {}


    this.style = 'fillColor=#eac133;strokeColor=#6666aa;lineWidth=2;rounded=2;fontColor=#000000;';
    this.cirStyleStart = 'shape=ellipse;fillColor=#ffddaa;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;';
    this.cirStyleTerminate = 'shape=ellipse;fillColor=#1133aa;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;';
    this.cirStyleEnd = 'shape=ellipse;fillColor=#112233;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#ffffff;';
    this.cirStyleAsk = 'shape=ellipse;fillColor=#eedd33;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;';
    this.cirStylePrompt = 'shape=ellipse;fillColor=#33ddaa;strokeColor=#3366aa;lineWidth=2;rounded=2;fontColor=#000000;';

    this.styles = {}
    this.styles[-100] = this.cirStylePrompt;
    this.styles[-101] = this.cirStyleAsk;
    this.styles[-102] = this.cirStyleEnd;
    this.styles[-103] = this.cirStyleTerminate;
    this.styles[-104] = this.cirStyleStart;

    this.styleTypes = {}
    this.styleTypes['prompt'] = -100;
    this.styleTypes['ask'] = -101;
    this.styleTypes['over'] = -102;
    this.styleTypes['terminate'] = -103;
    this.styleTypes['start'] = -104;

    if (!st) {
        if (!type) type = 'agent';
        if (!this.styleTypes.hasOwnProperty(type)) {
            st = this.style;
        } else {
            var id = this.styleTypes[type];
            st = this.styles[id];
        }
    }


    this.init(st);
}

Style.prototype.type = function () {
    return this.get('type');
}

Style.prototype.agent = function () {
    return this.get('agent');
}

Style.prototype.toString = function () {
    var str = '';
    for (var k in this.dc) {
        str += k + '=' + this.dc[k] + ';'
    }

    return str;
}

Style.prototype.get = function (k) {
    if (!k) return null;
    if (!this.dc.hasOwnProperty(k)) return null;
    return this.dc[k];
}

Style.prototype.set = function (k, v) {
    if (!k) return;
    this.dc[k] = v;
}

Style.prototype.remove = function (k) {
    if (!k) return null;

    if (!this.dc.hasOwnProperty(k)) return null;
    delete this.dc[k];
}

Style.prototype.init = function (str) {
    var segs = str.split(';');
    for (var i = 0; i < segs.length; i++) {
        var seg = segs[i]
        var kv = seg.split('=');
        var k = kv[0];
        var v = kv[1];
        this.dc[k] = v;
    }
}


var nodeEditPanel = new NodeEditPanel();

function putMod(e) {
    dropDownMod(e);
    var selections = graph.getSelectionCells();
    if (!selections || selections.length == 0) {
        nodeEditPanel.setCurrent(null);
        return;
    }

    var first = selections[0]
    nodeEditPanel.setCurrent(first);

    return false;
}

function dropDownMod(e) {
    if (current.Id == 0) return;
    var x = e.x - document.getElementById('models').clientWidth;
    var y = e.y - document.getElementById('tools').clientHeight;
    var name = current.Name;
    var st = new Style(null, current.type);
    st.set('type', current.type);
    st.set('agent', current.Id);
    var s = st.toString();
    var vetex = graph.insertVertex(parent, null, name, x, y, 100, 40, s);
    if (vetex) graph.selectCells([vetex]);
    current.Id = 0;
}


function removeGrap() {
    var nodes = graph.getSelectionCells(parent);
    if (nodes && nodes.length > 0) {
        for (var i = 0; i < nodes.length; i++) {
            var node = nodes[i];
            if (node.vertex && node.value == '开始') {
                nodes.splice(i, 1)
                i--;
            }
        }
    }
    graph.removeCells(nodes);
}

function onSelectEdge() {
    var nodes = graph.getSelectionCells(parent);
    if (!nodes) return;
    if (nodes.length == 0) return;

    var edgeStyleEle = document.getElementById('edgeStyle');
    var edgeStyle = edgeStyleEle.value;

    for (var i = 0; i < nodes.length; i++) {
        var node = nodes[i];
        if (!node.edge) return;
        var st = new Style(node.style);
        st.dc = {};
        st.set('edgeStyle', edgeStyle);
        node.style = st.toString();
    }

    graph.refresh();
}

function connect() {
    var nodes = graph.getSelectionCells(parent);
    if (!nodes) return;

    var datas = [];
    var last = null;
    for (var i = 0; i < nodes.length; i++) {
        var node = nodes[i];
        if (node.vertex) {
            if (last == null) {
                last = node;
                continue;
            } else {
                try {
                    var ltype = new Style(last.style).type()
                    var ntype = new Style(node.style).type()

                    if (ltype != 'ask' && ntype != 'ask') {
                        if (ltype == ntype) {
                            //相同类型，不能连线
                            continue;
                        }
                    }


                    if (ltype == 'agent' && ntype != 'ask') return;
                    if (ntype == 'agent' && ltype != 'ask') return;


                    if (ltype == 'prompt' && ntype != 'ask') continue;
                    if (ntype == 'prompt' && ltype != 'ask') continue;

                    var from = last;
                    var to = node;
                    if ((ltype == 'ask' && ntype == 'prompt')
                        || (ltype == 'terminate' && ntype != 'terminate')
                        || (ltype == 'over' && ntype != 'over')
                        || (ltype != 'start' && ntype == 'start')
                        || (ltype == 'ask' && ntype == 'agent')
                    ) {
                        from = node;
                        to = last;
                    }

                    var value = '下一步'
                    if (ltype == 'prompt' || ntype == 'prompt') value = '角色';
                    else if (ltype == 'agent' && ntype == 'ask') value = '执行者';
                    else if (ltype == 'ask' && ntype == 'agent') value = '执行者';


                    graph.insertEdge(parent, null, value, from, to, lineStyle);
                    last = node;
                } catch (e) { } finally {
                    last = node;
                }
            }
        }
    }
}


function loadConfig() {
    fetch('http://local.res/config', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            id: document.getElementById('flowId').value,
        })
    }).then(resp => {
        resp.json().then(d => {
            if (d.Success) {

                if (!(d.Data && d.Data.vertices && d.Data.vertices.length > 0)) {
                    var st = new Style(null, 'start');
                    st.set('agent', '-104');
                    st.set('type', 'start');
                    var s = st.toString();
                    graph.insertVertex(parent, null, '开始', 100, 100, 40, 40, s);
                }
                customDeserialize(graph, d.Data);
            }
        })
    }).catch(e => {
        alert('读取失败')
    })
}

function save() {
    var model = graph.getModel();
    var data = customSerialize(model);
    fetch('http://local.res/save', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: data
    }).then(resp => {
        resp.json().then(d => {
            if (d.Success) {
                alert('保存成功');
            }
        })
    }).catch(e => {
        alert('保存失败')
    })
}

function customSerialize(model) {
    const serializedData = {
        id: document.getElementById('flowId').value,
        vertices: [],
        edges: []
    };

    for (var k in model.cells) {
        var v = model.cells[k];
        if (v.vertex) {
            var st = new Style(v.style);
            var type = st.get('type');
            var agent = st.get('agent')
            var geo = v.geometry;
            var nd = nodeMap.hasOwnProperty(v.id) ? nodeMap[v.id] : null;
            var data = {
                id: v.id,
                value: v.value,
                height: geo.height,
                width: geo.width,
                x: geo.x,
                y: geo.y,
                type: type,
                agent: agent,
                prompt: !nd ? '' : nd.prompt,
                degree: !nd ? '' : nd.degree,
                paths: !nd ? [] : nd.paths,
            }

            serializedData.vertices.push(data);
        }
        else if (v.edge) {
            var data = {
                id: v.id,
                value: v.value,
                src: v.source.id,
                target: v.target.id,
                style: v.style
            }

            serializedData.edges.push(data);
        }
    }

    return JSON.stringify(serializedData);
}

function customDeserialize(graph, loadedData) {

    // 2. 预先创建所有顶点 (Vertices)
    const vertexMap = new Map(); // 使用 Map 存储 ID -> Vertex 实例

    for (var i = 0; i < loadedData.vertices.length; i++) {
        var vData = loadedData.vertices[i];
        // 创建一个新的顶点实例
        //  graph.insertVertex(parent, null, name, x, y, 100, 40, null);
        var st = new Style(null, vData.type);
        st.set('type', vData.type);
        st.set('agent', vData.agent)
        s = st.toString();
        const newVertex = graph.insertVertex(
            parent,
            vData.id,
            vData.value,
            vData.x,
            vData.y,
            vData.width,
            vData.height,
            s
        );
        vertexMap.set(vData.id + '', newVertex);

        var node = new Node();
        node.prompt = vData.prompt;
        node.paths = vData.paths;
        node.degree = vData.degree;
        nodeMap[vData.id] = node;
    }

    // 3. 遍历边，建立连接 (Edges)
    for (var i = 0; i < loadedData.edges.length; i++) {
        var eData = loadedData.edges[i];
        const sourceVertex = vertexMap.get(eData.src + '');
        const targetVertex = vertexMap.get(eData.target + '');

        if (sourceVertex && targetVertex) {
            // 插入边时，使用预先创建的顶点实例
            //graph.insertEdge(parent, null, '关联', last, node, '');
            graph.insertEdge(parent, eData.id, eData.value, sourceVertex, targetVertex, eData.style || '');
        }
    }

}


var agentEle = document.getElementById('agent');
var promptEle = document.getElementById('prompt');
var nameEle = document.getElementById('node');
var attributeEle = document.getElementById('attribute');
var pathsEle = document.getElementById('paths');
var degreeEle = document.getElementById('degree');
var nodeMap = {};

function Node() {
    this.id;
    this.name;
    this.prompt;
    this.agent;
    this.degree;
    this.paths = [];
}

function NodeEditPanel() {
    this.node = null;
}

NodeEditPanel.prototype.clear = function (nd) {
    agentEle.value = null;
    promptEle.value = '';
    nameEle.innerText = '';
}

NodeEditPanel.prototype.hide = function () {
    attributeEle.style.display = 'none'
}

NodeEditPanel.prototype.show = function () {
    attributeEle.style.display = 'block'
    this.showEdge();
}

NodeEditPanel.prototype.showEdge = function () {
    var vxs = document.querySelector('.vertex');
    for (var i = 0; i < vxs.length; i++) {
        vxs[i].style.display = 'block';
    }
}

NodeEditPanel.prototype.hideEdge = function () {
    var vxs = document.querySelector('.vertex');
    for (var i = 0; i < vxs.length; i++) {
        vxs[i].style.display = 'none';
    }
}

NodeEditPanel.prototype.setCurrent = function (nd) {
    this.clear();
    this.node = nd;
    if (!this.node) {
        this.hide();
        return;
    }

    if (this.node.edge) {
        this.hide();
        return;
    }

    var st = new Style(this.node.style);
    var agent = st.agent();
    var type = st.type();
    if (type != 'ask') {
        this.hide();
        return;
    }

    this.show();

    var node = new Node();
    node.id = nd.id;
    node.name = nd.value;
    node.agent = agent;
    if (nodeMap.hasOwnProperty(nd.id)) {
        node.prompt = nodeMap[nd.id].prompt;
        node.paths = nodeMap[nd.id].paths || [];
        node.degree = nodeMap[nd.id].degree || '100';
    }

    nodeMap[nd.id] = node;


    nameEle.innerText = this.node.value;
    promptEle.value = node.prompt;
    agentEle.value = agent;
    degreeEle.value = node.degree;

    this.setPaths(node);
}

NodeEditPanel.prototype.setPaths = function (info) {
    pathsEle.innerHTML = '';

    if (!this.node) return;
    var es = this.node.edges;
    if (!es || es.length == 0) return;

    var list = []
    for (var i = 0; i < es.length; i++) {
        var e = es[i];
        if (e.target.id != this.node.id) continue;
        list.push(e)
    }

    for (var i = 0; i < list.length; i++) {
        var e = list[i]
        var option = document.createElement('div')
       
        var input = document.createElement('input');
        input.className = 'path-input'; 
        input.type = 'checkbox';
        var attr = document.createAttribute('data-id');
        attr.value = e.id;
        input.setAttributeNode(attr);
        input.checked = false;
        for (var j = 0; j < info.paths.length; j++) {
            var path = info.paths[j];
            if (path === e.id) {
                input.checked = true;
                break;
            }
        }

        option.appendChild(input);

        var label = document.createElement('label')
        label.style.borderBottom = 'solid 1px whitesmoke'
        label.innerHTML = "[" + e.source.value + "] -> [" + e.value + "]";
        option.appendChild(label);


        pathsEle.appendChild(option);
    }
}


NodeEditPanel.prototype.confirm = function () {
    if (!this.node) return;

    var st = new Style(this.node.style);
    st.set('agent', agentEle.value);
    this.node.style = st.toString();

    var nd = nodeMap[this.node.id];
    nd.prompt = promptEle.value;
    nd.degree = degreeEle.value;
    nd.paths = [];
    var chks = document.querySelectorAll('.path-input');
    for (var i = 0; i < chks.length; i++) {
        var chk = chks[i];
        if (chk.checked) {
            nd.paths.push(chk.getAttribute('data-id'));
        }
    }

    graph.clearSelection();
    this.clear();
    this.hide();
}


function onConfirmNode(e) {
    nodeEditPanel.confirm();
}

nodeEditPanel.setCurrent(null);
