import { useState } from 'react';
import { Card, Table, Button, Tag, Modal, Select } from 'antd';
import { ShakeOutlined, PlayCircleOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { documentApi } from '@/api/document';
import dayjs from 'dayjs';

const Distillation = () => {
  const [selectedDoc, setSelectedDoc] = useState<string>('');
  const [isModalVisible, setIsModalVisible] = useState(false);
  const queryClient = useQueryClient();

  const { data: documents } = useQuery({
    queryKey: ['documents'],
    queryFn: () => documentApi.list({ page_size: 100 }),
  });

  const { data: knowledgePoints } = useQuery({
    queryKey: ['knowledge-points', selectedDoc],
    queryFn: () => documentApi.getKnowledgePoints(selectedDoc),
    enabled: !!selectedDoc,
  });

  const distillMutation = useMutation({
    mutationFn: (id: string) => documentApi.distill(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['knowledge-points', selectedDoc] });
      setIsModalVisible(false);
    },
  });

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">知识蒸馏</h1>

      <Card className="mb-6">
        <div className="flex items-center gap-4">
          <Select
            placeholder="选择文档"
            style={{ width: 300 }}
            options={documents?.data.map((doc: { id: string; title: string }) => ({
              value: doc.id,
              label: doc.title,
            }))}
            onChange={(value) => setSelectedDoc(value)}
          />
          <Button
            type="primary"
            icon={<PlayCircleOutlined />}
            onClick={() => setIsModalVisible(true)}
            disabled={!selectedDoc}
          >
            触发蒸馏
          </Button>
        </div>
      </Card>

      {selectedDoc && knowledgePoints && (
        <Card
          title={
            <span className="flex items-center gap-2">
              <ShakeOutlined />
              知识要点列表
            </span>
          }
        >
          <Table
            dataSource={knowledgePoints}
            columns={[
              {
                title: '内容',
                dataIndex: 'content',
                key: 'content',
                ellipsis: true,
              },
              {
                title: '置信度',
                dataIndex: 'confidence',
                key: 'confidence',
                render: (confidence: number) => (
                  <Tag color={confidence > 0.8 ? 'green' : confidence > 0.5 ? 'orange' : 'red'}>
                    {(confidence * 100).toFixed(1)}%
                  </Tag>
                ),
              },
              {
                title: '关键词',
                dataIndex: 'keywords',
                key: 'keywords',
                render: (keywords: string[]) => (
                  <div className="flex flex-wrap gap-1">
                    {keywords.map((kw, idx) => (
                      <Tag key={idx}>{kw}</Tag>
                    ))}
                  </div>
                ),
              },
              {
                title: '创建时间',
                dataIndex: 'created_at',
                key: 'created_at',
                render: (date: string) => dayjs(date).format('YYYY-MM-DD HH:mm'),
              },
            ]}
            rowKey="id"
            pagination={{ pageSize: 10 }}
          />
        </Card>
      )}

      <Modal
        title="确认蒸馏"
        visible={isModalVisible}
        onCancel={() => setIsModalVisible(false)}
        onOk={() => distillMutation.mutate(selectedDoc)}
        confirmLoading={distillMutation.isPending}
      >
        <p>确定要对该文档进行知识蒸馏吗？</p>
        <p className="text-gray-500 text-sm mt-2">蒸馏过程可能需要一些时间，请耐心等待。</p>
      </Modal>
    </div>
  );
};

export default Distillation;
