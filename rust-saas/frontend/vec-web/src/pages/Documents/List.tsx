import { useState } from 'react';
import {
  Card,
  Table,
  Button,
  Tag,
  Modal,
  Form,
  Input,
  Select,
  Space,
  Popconfirm,
} from 'antd';
import { FileTextOutlined, PlusOutlined, EditOutlined, DeleteOutlined, EyeOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { documentApi } from '@/api/document';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';

const DocumentList = () => {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [form] = Form.useForm();
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const { data: documents } = useQuery({
    queryKey: ['documents'],
    queryFn: () => documentApi.list({ page_size: 50 }),
  });

  const createMutation = useMutation({
    mutationFn: documentApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] });
      setIsModalVisible(false);
      form.resetFields();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: { title?: string; content?: string; visibility?: string } }) =>
      documentApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] });
      setIsModalVisible(false);
      form.resetFields();
      setEditingId(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: documentApi.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['documents'] }),
  });

  const handleEdit = (record: { id: string; title: string; content: string; visibility: string }) => {
    setEditingId(record.id);
    form.setFieldsValue({
      title: record.title,
      content: record.content,
      visibility: record.visibility,
    });
    setIsModalVisible(true);
  };

  const handleSubmit = () => {
    form.validateFields().then((values) => {
      if (editingId) {
        updateMutation.mutate({ id: editingId, data: values });
      } else {
        createMutation.mutate({ ...values, document_type: 'text' });
      }
    });
  };

  const columns = [
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
      ellipsis: true,
    },
    {
      title: '类型',
      dataIndex: 'document_type',
      key: 'document_type',
      render: (type: string) => (
        <Tag color={type === 'text' ? 'blue' : 'green'}>{type}</Tag>
      ),
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
      title: '可见性',
      dataIndex: 'visibility',
      key: 'visibility',
      render: (visibility: string) => (
        <Tag
          color={
            visibility === 'public'
              ? 'green'
              : visibility === 'private'
              ? 'red'
              : 'orange'
          }
        >
          {visibility}
        </Tag>
      ),
    },
    {
      title: '字数',
      dataIndex: 'word_count',
      key: 'word_count',
    },
    {
      title: '版本',
      dataIndex: 'version',
      key: 'version',
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => dayjs(date).format('YYYY-MM-DD'),
    },
    {
      title: '操作',
      key: 'actions',
      render: (_: unknown, record: { id: string; title: string; content: string; visibility: string }) => (
        <Space>
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => navigate(`/documents/${record.id}`)}
          >
            查看
          </Button>
          <Button
            type="link"
            icon={<EditOutlined />}
            onClick={() => handleEdit(record)}
          >
            编辑
          </Button>
          <Popconfirm
            title="确定删除？"
            onConfirm={() => deleteMutation.mutate(record.id)}
          >
            <Button type="link" danger icon={<DeleteOutlined />}>
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">文档管理</h1>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => {
            setEditingId(null);
            form.resetFields();
            setIsModalVisible(true);
          }}
        >
          新建文档
        </Button>
      </div>

      <Card
        title={
          <span className="flex items-center gap-2">
            <FileTextOutlined />
            文档列表
          </span>
        }
      >
        <Table
          dataSource={documents?.data}
          columns={columns}
          rowKey="id"
          pagination={{ pageSize: 10 }}
        />
      </Card>

      <Modal
        title={editingId ? '编辑文档' : '新建文档'}
        visible={isModalVisible}
        onCancel={() => {
          setIsModalVisible(false);
          setEditingId(null);
          form.resetFields();
        }}
        onOk={handleSubmit}
        confirmLoading={createMutation.isPending || updateMutation.isPending}
      >
        <Form form={form} layout="vertical">
          <Form.Item name="title" label="标题" rules={[{ required: true }]}>
            <Input />
          </Form.Item>
          <Form.Item name="content" label="内容" rules={[{ required: true }]}>
            <Input.TextArea rows={4} />
          </Form.Item>
          <Form.Item name="visibility" label="可见性">
            <Select
              options={[
                { value: 'public', label: '公开' },
                { value: 'private', label: '私有' },
                { value: 'restricted', label: '受限' },
              ]}
              defaultValue="public"
            />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default DocumentList;
