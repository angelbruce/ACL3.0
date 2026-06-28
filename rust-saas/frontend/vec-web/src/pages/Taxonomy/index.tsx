import { useState } from 'react';
import { Card, Tree, Button, Modal, Form, Input, Tag } from 'antd';
import { TagsOutlined, PlusOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { taxonomyApi } from '@/api/taxonomy';
import type { Category } from '@/types/taxonomy';

interface CategoryItem extends Category {
  children?: CategoryItem[];
}

interface TreeNode {
  title: React.ReactNode;
  key: string;
  children?: TreeNode[];
}

const Taxonomy = () => {
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [form] = Form.useForm();
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const { data: categories } = useQuery({
    queryKey: ['categories'],
    queryFn: taxonomyApi.categories,
  });

  const createMutation = useMutation({
    mutationFn: taxonomyApi.createCategory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      setIsModalVisible(false);
      form.resetFields();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: { name?: string } }) =>
      taxonomyApi.updateCategory(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
      setIsModalVisible(false);
      form.resetFields();
      setEditingId(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: taxonomyApi.deleteCategory,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['categories'] }),
  });

  const handleEdit = (item: CategoryItem) => {
    setEditingId(item.id);
    form.setFieldsValue({ name: item.name });
    setIsModalVisible(true);
  };

  const handleSubmit = () => {
    form.validateFields().then((values) => {
      if (editingId) {
        updateMutation.mutate({ id: editingId, data: values });
      } else {
        createMutation.mutate(values);
      }
    });
  };

  const renderTreeNodes = (list: CategoryItem[]): TreeNode[] => {
    return list.map((item) => ({
      title: (
        <div className="flex items-center gap-2">
          <span>{item.name}</span>
          <Tag color="gray">{item.document_count} 文档</Tag>
          <Button type="link" icon={<EditOutlined />} onClick={() => handleEdit(item)} />
          <Button type="link" danger icon={<DeleteOutlined />} onClick={() => deleteMutation.mutate(item.id)} />
        </div>
      ),
      key: item.id,
      children: item.children ? renderTreeNodes(item.children) : undefined,
    }));
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">知识分类</h1>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => {
            setEditingId(null);
            form.resetFields();
            setIsModalVisible(true);
          }}
        >
          新建分类
        </Button>
      </div>

      <Card
        title={
          <span className="flex items-center gap-2">
            <TagsOutlined />
            分类树
          </span>
        }
      >
        <Tree
          treeData={categories ? renderTreeNodes(categories) : []}
          defaultExpandAll
        />
      </Card>

      <Modal
        title={editingId ? '编辑分类' : '新建分类'}
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
          <Form.Item name="name" label="名称" rules={[{ required: true }]}>
            <Input />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default Taxonomy;
