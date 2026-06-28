import { useState } from 'react';
import { Card, Select, Table, Button, Tag, Modal } from 'antd';
import { HistoryOutlined, RollbackOutlined, CompressOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { documentApi } from '@/api/document';
import { versionApi } from '@/api/version';
import type { Version } from '@/types/version';
import dayjs from 'dayjs';

const VersionPage = () => {
  const [selectedDoc, setSelectedDoc] = useState<string>('');
  const queryClient = useQueryClient();

  const { data: documents } = useQuery({
    queryKey: ['documents'],
    queryFn: () => documentApi.list({ page_size: 100 }),
  });

  const { data: versions } = useQuery({
    queryKey: ['versions', selectedDoc],
    queryFn: () => versionApi.list(selectedDoc),
    enabled: !!selectedDoc,
  });

  const rollbackMutation = useMutation({
    mutationFn: (data: { documentId: string; versionNumber: number }) =>
      versionApi.rollback(data.documentId, data.versionNumber),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['versions', selectedDoc] }),
  });

  const handleRollback = (versionNumber: number) => {
    Modal.confirm({
      title: '确认回滚',
      content: `确定要回滚到版本 ${versionNumber} 吗？`,
      onOk: () => rollbackMutation.mutate({ documentId: selectedDoc, versionNumber }),
    });
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">版本管理</h1>

      <Card className="mb-6">
        <Select
          placeholder="选择文档"
          style={{ width: 300 }}
          options={documents?.data.map((doc: { id: string; title: string; version: number }) => ({
            value: doc.id,
            label: `${doc.title} (v${doc.version})`,
          }))}
          onChange={(value) => setSelectedDoc(value)}
        />
      </Card>

      {selectedDoc && versions && (
        <Card
          title={
            <span className="flex items-center gap-2">
              <HistoryOutlined />
              版本列表
            </span>
          }
          extra={<Button icon={<CompressOutlined />}>对比版本</Button>}
        >
          <Table
            dataSource={versions}
            columns={[
              {
                title: '版本号',
                dataIndex: 'version_number',
                key: 'version_number',
                render: (num: number) => <Tag color="blue">v{num}</Tag>,
              },
              {
                title: '标题',
                dataIndex: 'title',
                key: 'title',
              },
              {
                title: '状态',
                dataIndex: 'status',
                key: 'status',
                render: (status: string) => (
                  <Tag
                    color={
                      status === 'completed'
                        ? 'success'
                        : status === 'processing'
                        ? 'processing'
                        : 'error'
                    }
                  >
                    {status}
                  </Tag>
                ),
              },
              {
                title: '变更摘要',
                dataIndex: 'change_summary',
                key: 'change_summary',
                ellipsis: true,
              },
              {
                title: '创建时间',
                dataIndex: 'created_at',
                key: 'created_at',
                render: (date: string) => dayjs(date).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: '操作',
                key: 'actions',
                render: (_: unknown, record: Version) => (
                  <div className="flex gap-2">
                    <Button
                      type="link"
                      icon={<RollbackOutlined />}
                      onClick={() => handleRollback(record.version_number)}
                    >
                      回滚
                    </Button>
                  </div>
                ),
              },
            ]}
            rowKey="id"
            pagination={{ pageSize: 10 }}
          />
        </Card>
      )}
    </div>
  );
};

export default VersionPage;
