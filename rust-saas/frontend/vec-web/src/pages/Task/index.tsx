import { Card, Table, Tag, Progress, Button } from 'antd';
import { ClockCircleOutlined, PauseCircleOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import { useQuery } from '@tanstack/react-query';
import { taskApi } from '@/api/task';
import { documentApi } from '@/api/document';
import type { Task } from '@/types/task';
import dayjs from 'dayjs';

const TaskPage = () => {
  const { data: tasks } = useQuery({
    queryKey: ['tasks'],
    queryFn: () => taskApi.list(),
  });

  const { data: documents } = useQuery({
    queryKey: ['documents'],
    queryFn: () => documentApi.list({ page_size: 100 }),
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <ClockCircleOutlined className="text-gray-400" />;
      case 'running':
        return <PauseCircleOutlined className="text-blue-500" />;
      case 'completed':
        return <CheckCircleOutlined className="text-green-500" />;
      case 'failed':
        return <CloseCircleOutlined className="text-red-500" />;
      default:
        return null;
    }
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">任务管理</h1>

      <Card
        title={
          <span className="flex items-center gap-2">
            <ClockCircleOutlined />
            任务列表
          </span>
        }
      >
        <Table<Task>
          dataSource={tasks}
          columns={[
            {
              title: '任务ID',
              dataIndex: 'id',
              key: 'id',
              ellipsis: true,
            },
            {
              title: '任务类型',
              dataIndex: 'task_type',
              key: 'task_type',
              render: (type: string) => <Tag color="blue">{type}</Tag>,
            },
            {
              title: '关联文档',
              key: 'document',
              render: (_: unknown, record: Task) => {
                const doc = documents?.data.find((d: { id: string }) => d.id === record.document_id);
                return doc?.title || '未知';
              },
            },
            {
              title: '状态',
              dataIndex: 'status',
              key: 'status',
              render: (status: string) => (
                <span className="flex items-center gap-2">
                  {getStatusIcon(status)}
                  <Tag
                    color={
                      status === 'completed'
                        ? 'success'
                        : status === 'running'
                        ? 'processing'
                        : status === 'failed'
                        ? 'error'
                        : 'default'
                    }
                  >
                    {status}
                  </Tag>
                </span>
              ),
            },
            {
              title: '进度',
              dataIndex: 'progress',
              key: 'progress',
              render: (progress: number) => <Progress percent={progress} size="small" />,
            },
            {
              title: '创建时间',
              dataIndex: 'created_at',
              key: 'created_at',
              render: (date: string) => dayjs(date).format('YYYY-MM-DD HH:mm'),
            },
            {
              title: '更新时间',
              dataIndex: 'updated_at',
              key: 'updated_at',
              render: (date: string) => dayjs(date).format('YYYY-MM-DD HH:mm'),
            },
            {
              title: '操作',
              key: 'actions',
              render: (_: unknown, record: Task) => (
                <Button
                  type="link"
                  danger
                  disabled={record.status === 'completed'}
                >
                  取消
                </Button>
              ),
            },
          ]}
          rowKey="id"
          pagination={{ pageSize: 10 }}
        />
      </Card>
    </div>
  );
};

export default TaskPage;
