import { useState } from 'react';
import { Card, Select, Button, Tag, Table, Modal, Radio } from 'antd';
import { IeOutlined, PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { documentApi } from '@/api/document';
import { boundaryApi } from '@/api/boundary';
import type { Share } from '@/types/boundary';

const Boundary = () => {
  const [selectedDoc, setSelectedDoc] = useState<string>('');
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [visibility, setVisibility] = useState<'public' | 'private' | 'restricted'>('public');
  const queryClient = useQueryClient();

  const { data: documents } = useQuery({
    queryKey: ['documents'],
    queryFn: () => documentApi.list({ page_size: 100 }),
  });

  const { data: shares } = useQuery({
    queryKey: ['shares', selectedDoc],
    queryFn: () => boundaryApi.shares(selectedDoc),
    enabled: !!selectedDoc,
  });

  const setVisibilityMutation = useMutation({
    mutationFn: (data: { documentId: string; visibility: typeof visibility }) =>
      boundaryApi.setVisibility(data.documentId, { visibility: data.visibility }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] });
      setIsModalVisible(false);
    },
  });

  const deleteShareMutation = useMutation({
    mutationFn: boundaryApi.deleteShare,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['shares', selectedDoc] }),
  });

  const handleDeleteShare = (id: string) => {
    Modal.confirm({
      title: '确认删除',
      content: '确定要删除该共享吗？',
      onOk: () => deleteShareMutation.mutate(id),
    });
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">知识边界</h1>

      <Card className="mb-6">
        <div className="flex items-center gap-4">
          <Select
            placeholder="选择文档"
            style={{ width: 300 }}
            options={documents?.data.map((doc: { id: string; title: string; visibility: string }) => ({
              value: doc.id,
              label: `${doc.title} (${doc.visibility})`,
            }))}
            onChange={(value) => setSelectedDoc(value)}
          />
          <Button
            type="primary"
            icon={<IeOutlined />}
            onClick={() => setIsModalVisible(true)}
            disabled={!selectedDoc}
          >
            设置可见性
          </Button>
        </div>
      </Card>

      {selectedDoc && shares && (
        <Card
          title={
            <span className="flex items-center gap-2">
              <IeOutlined />
              共享列表
            </span>
          }
          extra={<Button icon={<PlusOutlined />}>添加共享</Button>}
        >
          <Table
            dataSource={shares}
            columns={[
              {
                title: '用户/组',
                key: 'user',
                render: (record: Share) => record.user_id || record.group_id || '未知',
              },
              {
                title: '权限',
                dataIndex: 'permission',
                key: 'permission',
                render: (perm: string) => (
                  <Tag color={perm === 'write' ? 'orange' : 'blue'}>{perm}</Tag>
                ),
              },
              {
                title: '操作',
                key: 'actions',
                render: (_: unknown, record: Share) => (
                  <Button
                    type="link"
                    danger
                    icon={<DeleteOutlined />}
                    onClick={() => handleDeleteShare(record.id)}
                  >
                    删除
                  </Button>
                ),
              },
            ]}
            rowKey="id"
          />
        </Card>
      )}

      <Modal
        title="设置可见性"
        visible={isModalVisible}
        onCancel={() => setIsModalVisible(false)}
        onOk={() => setVisibilityMutation.mutate({ documentId: selectedDoc, visibility })}
        confirmLoading={setVisibilityMutation.isPending}
      >
        <Radio.Group
          value={visibility}
          onChange={(e) => setVisibility(e.target.value)}
          className="flex flex-col gap-3"
        >
          <Radio value="public">公开 - 所有人可见</Radio>
          <Radio value="private">私有 - 仅自己可见</Radio>
          <Radio value="restricted">受限 - 指定用户可见</Radio>
        </Radio.Group>
      </Modal>
    </div>
  );
};

export default Boundary;
