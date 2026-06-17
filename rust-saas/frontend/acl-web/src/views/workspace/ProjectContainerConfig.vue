<script lang="ts" setup>
import { ref, onMounted } from 'vue'
import type {
    Project, ProjectChatMessage, ProjectContainerConfig, LlmRequest
} from '@/types'
import { useWorkspaceStore } from '@/stores/workspace'
import { llmService, StreamResponse } from '@/api';
import { FileWarningIcon, Settings2Icon, Terminal, } from 'lucide-vue-next';

const workspaceStore = useWorkspaceStore();
const activeTab = ref(1);
const activeTab1 = ref('basicConfig');
const dockerCompose = ref('');

interface Props {
    project: Project,
    messages: ProjectChatMessage[]
}

let containerConfigs = ref<ProjectContainerConfig[]>([]);
let props = defineProps<Props>();

onMounted(() => {
    if (props.project && props.project.id) {
        workspaceStore.getProjectContainerConfigs(props.project.id).then((data: ProjectContainerConfig[]) => {
            data = data || []
            data.forEach(x=>{
                x.image_name =  'app-debug-base:latest'
            })
            let compose = data.filter(x=>x.container_name==='docker-compose.yml') || []
            let configs = data.filter(x=>x.container_name!=='docker-compose.yml') || []

            if(compose.length > 0) dockerCompose.value = compose[0].environment || ''
            containerConfigs.value = configs || []
            if(containerConfigs.value.length > 0){
                activeTab.value = containerConfigs.value[0].id;
            }
        })
    }
})


let streamingContent = ref('');
let isLlmRunning = ref(false);
const fetchBasicInfo = () => {
    try {
        isLlmRunning.value = true;
        let model_id = props.project.model_id;
        let agent_id = props.project.agent_id;
        if (!model_id && !agent_id) {
            isLlmRunning.value = false;
            return;
        }
        let msgs = props.messages.filter((msg) => msg.role !== 'user');
        if (msgs.length === 0) {
            isLlmRunning.value = false;
            return;
        }

         msgs.push({
            id: -1,
            role: 'system',
            project_id: props.project.id,
            content: '你是一名高级软件工程师，负责对软件进行调试和优化。简单干练，快速响应，不做任何解释，不做任何推理，不做任何假设，不做任何猜测，不做任何建议，也不做多余思考。',
            created_at: msgs[0].created_at,
        })
        msgs.push({
            id: 0,
            role: 'user',
            project_id: props.project.id,
            content: '代码要进入到docker环境下进行编译调试，如实对所有的程序代码进行筛选，获取编译调试程序所需要容器的所有相关信息，组织成jsons以及docker-compose.yml，输出内容没有先后次序约束:\n'
                +'a) 相关信息包括：端口、卷（目录）、网络信息、程序调试启动的命令、docker环境变量、工作目录、最低内存、最低cpu占，并**严格按照如下格式返回**，其中JSONs的SCHEMA格式为：'
                + '```jsons [{"published_ports":string,"volumes":string,"container_name":string,"environment":string,"command":string,"tags":string,"working_dir":string,"memory_usage":string,"cpu_usage":string,"image_name":string}]```'
                +'b) docker-compose.yml，必须是将以上收集的容器信息运行起来的所有内容，并**严格**按照yaml的格式进行书写，输出格式为：```yaml  <YAML内容>```。'
                ,
            created_at: msgs[0].created_at,
        });

        let chatRequest: LlmRequest = {
            model_id: model_id!,
            messages: msgs,
            stream: true,
        }
        streamingContent.value = '';
        llmService.chatStream(chatRequest,
            (data: StreamResponse) => {
                if (data.finish_reason && data.finish_reason === 'stop') {
                    fetchJson();
                    isLlmRunning.value = false;
                } else {
                    streamingContent.value += data.content || '';
                }
            },
            (error: Error) => {
                console.error('[ERROR] Stream error:', error);
                isLlmRunning.value = false;
            }
        );
    } catch (error) {
        console.error('Error:', error);
        isLlmRunning.value = false;
    }
}


const fetchJson = () => {
    containerConfigs.value = [];
    let configs = [];
    let jsons = [...streamingContent.value.matchAll(/```([\s\S.]+?)\n\s*([\S\s.]+?)```/g)];
    if (jsons.length > 0) {
        for (let json of jsons) {
            let prefix = json[1];
            if(prefix.startsWith('json') || prefix.startsWith('JSON')) {
                let jsonStr = json[2];
                try {
                    let jsonObj = JSON.parse(jsonStr);
                    if (Array.isArray(jsonObj)) {
                        for (let item of jsonObj) {
                            configs.push(item);
                        }
                    } else {
                        configs.push(jsonObj);
                    }
                } catch (error) {
                    console.log('Error parsing JSON:', error);
                }
            } else if(prefix.startsWith('yml') || prefix.startsWith('yaml')) {
                console.log(json[2])
                dockerCompose.value = json[2]
            }
        }
    }

    for(let i = 0; i < configs.length; i++){
        configs[i].id = i;
    }

    if(configs.length > 0){
        activeTab.value = configs[0].id;
    } 

    containerConfigs.value = configs;
}



const saveContainerConfig = () => {
    let idx = 0;
    for (let config of containerConfigs.value) {
        idx ++;
        if(!config.id) config.id = idx;
        if(!config.project_id) config.project_id = props.project.id;
        if(!config.project_dir) config.project_dir = '';
        if(!config.image_name) config.image_name = '';
        if(!config.container_name) config.container_name = 'container_' + idx;
        if(!config.environment) config.environment = '';
        if(!config.command) config.command = '';
        if(!config.tags) config.tags = '';
        if(!config.working_dir) config.working_dir = '';
        if(!config.memory_usage) config.memory_usage = '';
        if(!config.cpu_usage) config.cpu_usage = '';
        if(!config.published_ports) config.published_ports = '';
        if(!config.volumes) config.volumes = '';
        if(!config.created_at) config.created_at = getNow();
        if(!config.updated_at) config.updated_at = getNow();
        if(!config.creator_id) config.creator_id = 0;

        config.environment = getString(config.environment)
        config.command = getString(config.command)
        config.image_name = getString(config.image_name)
        config.tags = getString(config.tags)
        config.working_dir = getString(config.working_dir)
        config.project_dir = getString(config.project_dir)
        config.memory_usage = getString(config.memory_usage)
        config.cpu_usage = getString(config.cpu_usage)
        config.published_ports = getString(config.published_ports)
        config.volumes = getString(config.volumes)
    }

    console.log(containerConfigs.value);
   

    let savedConfigs = [...containerConfigs.value];
     if(dockerCompose.value){
        savedConfigs.push({
            id: ++idx,
            project_dir: '',
            image_name: '',
            container_name: 'docker-compose.yml',
            project_id: props.project.id,
            environment: dockerCompose.value,
            command: '',
            tags: '',
            working_dir: '',
            memory_usage: '',
            cpu_usage: '',
            published_ports: '',
            volumes: '',
            created_at: getNow(),
            updated_at: getNow(),
            creator_id: 0,
        })
    }

    workspaceStore.saveProjectContainerConfigs(props.project.id, savedConfigs).then(() => {
        let data = workspaceStore.projectContainerConfigs ||[]
        let compose = data.filter(x=>x.container_name==='docker-compose.yml') || []
        let configs = data.filter(x=>x.container_name!=='docker-compose.yml') || []
        if(compose.length > 0) dockerCompose.value = compose[0].environment || ''
        containerConfigs.value = configs || []
        if(containerConfigs.value.length > 0){
            activeTab.value = containerConfigs.value[0].id;
        }
    })
}

const editableTabsValue = ref('')
const handleTabClick = ( tab : any, event: any) => {
    editableTabsValue.value = tab
}

const closeTab = (containerConfig: any) => {
    containerConfigs.value = containerConfigs.value.filter((item: any) => item.id !== containerConfig.id);
    for(let i = 0; i < containerConfigs.value.length; i++){
        let data = containerConfigs.value[i]
        console.log(editableTabsValue.value.paneName,data.id.toString());
        if (data.id.toString() === editableTabsValue.value.paneName+'') {

            containerConfigs.value.splice(i, 1)
            break;
        }
    }
}

const getNow = () => {
    let date = new Date();
    let s = '';
    s += date.getFullYear() + '-';
    s += (date.getMonth() + 1 < 10 ? '0' + (date.getMonth() + 1) : date.getMonth() + 1) + '-';
    s += (date.getDate() < 10 ? '0' + date.getDate() : date.getDate() )+ 'T';
    s += (date.getHours() < 10 ? '0' + date.getHours() : date.getHours() )+ ':';
    s += (date.getMinutes() < 10 ? '0' + date.getMinutes() : date.getMinutes() )+ ':';
    s += (date.getSeconds() < 10 ? '0' + date.getSeconds() : date.getSeconds() )+ '.' + date.getMilliseconds() * 1000;
    return s;
}

const getString = (data: any) => {
    var s = '';
    if(Array.isArray(data)){
        for(let item of data){
            s += getString(item) + ',';
        }
    } else if(typeof data === 'object'){
        for(let key in data){
            s += getString(data[key]);
        }
    } else {
        s += data;  
    }
    return s; 
}


const startContainer = () => {
    workspaceStore.startContainer(props.project.id)
}



</script>
<template class="flex flex-col gap-2">
    <div class="flex justify-start items-left my-2">
        <el-button @click="fetchBasicInfo()" :disabled="isLlmRunning" type="primary">提取信息</el-button>
        <el-button @click="saveContainerConfig()" type="info" v-if="containerConfigs.length > 0">更新配置</el-button>
        <el-button type="primary" v-if="containerConfigs.length > 0" @click="startContainer()">启动</el-button>
        <el-button type="warning" v-if="containerConfigs.length > 0">重新启动</el-button>
        <el-button type="danger" v-if="containerConfigs.length > 0">关闭容器</el-button>
    </div>
    <div v-if="containerConfigs.length === 0"  class="flex justify-start items-left py-4 text-surface-600 text-sm my-2 flex-row">
        <file-warning-icon class="mr-2" /><span v-if="!isLlmRunning" >暂无容器配置,请先提取信息进行配置。</span><span v-else>提取中，请稍后...</span>
    </div>

     <highlightjs v-if="isLlmRunning"
              class="w-full h-30 resize-none border-none outline-none bg-transparent text-surface-700 leading-relaxed text-base
              word-break-break-word
              overflow-wrap-break-word
              whitespace-pre-wrap
              overflow-auto
              leading-relaxed text-base
              " 
              style="font-family: 'Georgia', 'Times New Roman', serif;"
              :code="streamingContent" 
              autodetect />
    <div v-if="containerConfigs.length > 0 && !isLlmRunning" class="w-full ">
        <el-tabs type="card" class="w-full" @tab-remove="closeTab" @tab-click="handleTabClick" v-model="activeTab">
            <el-tab-pane :name="0" v-if="dockerCompose.length > 0">
                <template #label>
                    <span>docker-compose.yml</span>
                </template>
                 <highlightjs v-if="!isLlmRunning"
                    class="w-full resize-none border-none 
                    outline-none 
                    bg-transparent text-surface-700 leading-relaxed text-base
                    word-break-break-word
                    overflow-wrap-break-word
                    whitespace-pre-wrap
                    overflow-auto
                    leading-relaxed text-base
                    " 
                    style="font-family: 'Georgia', 'Times New Roman', serif;"
                    :code="dockerCompose" 
                    autodetect />
            </el-tab-pane>
            <el-tab-pane :name="containerConfig.id" v-for="containerConfig in containerConfigs" :key="containerConfig.id" closable @close="closeTab(containerConfig)">
                <template #label>
                    <span>{{ containerConfig.container_name }}</span>
                </template>
                <div class="w-full">
                    <el-tabs type="border-card" tab-position="top" v-model="activeTab1">
                        <el-tab-pane name="basicConfig"> 
                            <template #label>
                                <settings2-icon />
                                <span>基本设置</span>
                            </template>
                            <div class="flex flex-col gap-2 w-full">
                                <form>
                                    <el-form-item label="容器名称" prop="containerName">
                                        <input type="text" placeholder="容器名称" v-model="containerConfig.container_name"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="项目路径" prop="projectPath">
                                        <input type="text" placeholder="项目路径，linux路径"
                                            v-model="containerConfig.project_dir"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="发布端口" prop="port">
                                        <input type="text" placeholder="发布端口，多个端口用逗号隔开"
                                            v-model="containerConfig.published_ports"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="卷映射名" prop="volumeName">
                                        <input type="text" placeholder="卷映射名称" v-model="containerConfig.volumes"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="环境变量" prop="env">
                                        <textarea placeholder="环境变量，多个环境变量用逗号隔开" v-model="containerConfig.environment"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="启动命令" prop="command">
                                        <textarea placeholder="启动命令，多个命令用逗号隔开，最后一个为常驻命令"
                                            v-model="containerConfig.command"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="工作目录" prop="workDir">
                                        <input type="text" placeholder="工作目录" v-model="containerConfig.working_dir"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="容器标签" prop="containerTag">
                                        <input type="text" placeholder="容器标签" v-model="containerConfig.tags"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="容器内存" prop="memory">
                                        <input type="text" placeholder="容器内存，单位MB"
                                            v-model="containerConfig.memory_usage"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    <el-form-item label="容器CPU" prop="cpu">
                                        <input type="text" placeholder="容器CPU，单位核数" v-model="containerConfig.cpu_usage"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>
                                    
                                    <el-form-item label="容器镜像" prop="image">
                                        <input type="text" placeholder="容器镜像"
                                            readonly
                                            v-model="containerConfig.image_name"
                                            class="w-full px-3 py-2 bg-surface-100 border border-surface-200 rounded-lg  text-sm text-surface-600" />
                                    </el-form-item>


                                </form>
                            </div>
                        </el-tab-pane>
                        <el-tab-pane name="terminal">
                            <template #label>
                                <terminal />
                                <span>终端</span>
                            </template>
                            <div>
                                输入输出终端
                            </div>
                        </el-tab-pane>
                    </el-tabs>
                </div>
            </el-tab-pane>
        </el-tabs>

    </div>
</template>