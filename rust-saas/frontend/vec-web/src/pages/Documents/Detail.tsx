import { useParams, Link } from 'react-router-dom';
import { Card, Button, Tag, Descriptions, Divider } from 'antd';
import { ArrowLeftOutlined, ShakeOutlined } from '@ant-design/icons';
import { useQuery } from '@tanstack/react-query';
import { documentApi } from '@/api/document';
import dayjs from 'dayjs';

const DocumentDetail = () => {
  const { id } = useParams<{ id: string }>();

  const { data: document } = useQuery({
    queryKey: ['document', id],
    queryFn: () => documentApi.get(id!),
  });

  const { data: knowledgePoints } = useQuery({
    queryKey: ['knowledge-points', id],
    queryFn: () => documentApi.getKnowledgePoints(id!),
    enabled: !!id,
  });

  if (!document) {
    return <div>加载中...</div>;
  }

  return (
    <div>
      <div className="flex items-center gap-4 mb-6">
        <Button icon={<ArrowLeftOutlined />}>
          <Link to="/documents">返回列表</Link>
        </Button>
        <h1 className="text-2xl font-bold">{document.title}</h1>
      </div>

      <Card className="mb-6">
        <Descriptions bordered column={3}>
          <Descriptions.Item label="类型">
            <Tag color={document.document_type === 'text' ? 'blue' : 'green'}>
              {document.document_type}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="状态">
            <Tag
              color={
                document.status === 'completed'
                  ? 'success'
                  : document.status === 'processing'
                  ? 'processing'
                  : 'error'
              }
            >
              {document.status}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="可见性">
            <Tag
              color={
                document.visibility === 'public'
                  ? 'green'
                  : document.visibility === 'private'
                  ? 'red'
                  : 'orange'
              }
            >
              {document.visibility}
            </Tag>
          </Descriptions.Item>
          <Descriptions.Item label="版本">{document.version}</Descriptions.Item>
          <Descriptions.Item label="字数">{document.word_count}</Descriptions.Item>
          <Descriptions.Item label="分块数">{document.chunk_count}</Descriptions.Item>
          <Descriptions.Item label="创建时间" span={3}>
            {dayjs(document.created_at).format('YYYY-MM-DD HH:mm:ss')}
          </Descriptions.Item>
          <Descriptions.Item label="更新时间" span={3}>
            {dayjs(document.updated_at).format('YYYY-MM-DD HH:mm:ss')}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      <Card title="文档内容" className="mb-6">
        <div className="whitespace-pre-wrap text-gray-800 leading-relaxed">
          {document.content}
        </div>
      </Card>

      {knowledgePoints && knowledgePoints.length > 0 && (
        <Card
          title={
            <span className="flex items-center gap-2">
              <ShakeOutlined />
              知识要点
            </span>
          }
        >
          <div className="space-y-4">
            {knowledgePoints.map((point) => (
              <div key={point.id} className="p-4 bg-gray-50 rounded-lg">
                <p className="text-gray-800">{point.content}</p>
                <div className="flex items-center gap-4 mt-2">
                  <Tag color="blue">置信度: {(point.confidence * 100).toFixed(1)}%</Tag>
                  <div className="flex gap-2">
                    {point.keywords.map((keyword, idx) => (
                      <Tag key={idx}>{keyword}</Tag>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      <Divider />
      <div className="flex gap-2">
        <Button icon={<ShakeOutlined />}>触发知识蒸馏</Button>
        <Button>重新索引</Button>
      </div>
    </div>
  );
};

export default DocumentDetail;
