import { useState } from 'react';
import { Card, Input, Tree, Tag } from 'antd';
import { ForkOutlined, SearchOutlined } from '@ant-design/icons';
import { useQuery } from '@tanstack/react-query';
import { graphApi } from '@/api/graph';

const KnowledgeGraph = () => {
  const [searchQuery, setSearchQuery] = useState('');

  const { data: graphData } = useQuery({
    queryKey: ['knowledge-graph'],
    queryFn: graphApi.data,
  });

  const renderTreeNodes = (list: any[]) => {
    return list.map((item) => ({
      title: (
        <span>
          <Tag color="blue">{item.name}</Tag>
          <span className="ml-2">{item.entity_type}</span>
        </span>
      ),
      key: item.id,
      children: item.relations?.map((rel: any) => ({
        title: (
          <span>
            <Tag color="green">{rel.relation_type}</Tag>
            <span className="ml-2">{rel.target_entity_name}</span>
          </span>
        ),
        key: rel.id,
      })),
    }));
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">知识图谱</h1>

      <Card className="mb-6">
        <Input
          placeholder="搜索实体"
          prefix={<SearchOutlined />}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </Card>

      <Card
        title={
          <span className="flex items-center gap-2">
            <ForkOutlined />
            实体关系树
          </span>
        }
      >
        <Tree
          treeData={graphData?.entities ? renderTreeNodes(graphData.entities) : []}
          defaultExpandAll
        />
      </Card>
    </div>
  );
};

export default KnowledgeGraph;
