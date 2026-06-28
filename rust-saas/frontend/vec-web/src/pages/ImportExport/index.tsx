import { useState } from 'react';
import { Card, Upload, Button, Progress, message, Row, Col } from 'antd';
import { ImportOutlined, ExportOutlined, FileTextOutlined, ForkOutlined } from '@ant-design/icons';
import { useMutation } from '@tanstack/react-query';
import { importExportApi } from '@/api/import_export';

const ImportExport = () => {
  const [uploadProgress, setUploadProgress] = useState(0);
  const [isUploading, setIsUploading] = useState(false);

  const importMutation = useMutation({
    mutationFn: importExportApi.importDocuments,
    onMutate: () => {
      setIsUploading(true);
      setUploadProgress(0);
    },
    onSuccess: () => {
      message.success('导入成功');
      setIsUploading(false);
      setUploadProgress(100);
    },
    onError: () => {
      message.error('导入失败');
      setIsUploading(false);
      setUploadProgress(0);
    },
  });

  const handleImport = (files: File[]) => {
    if (files.length > 0) {
      importMutation.mutate(files);
    }
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">导入导出</h1>

      <Row gutter={16}>
        <Col span={12}>
          <Card
            title={
              <span className="flex items-center gap-2">
                <ImportOutlined />
                导入文档
              </span>
            }
          >
            <Upload.Dragger
              accept=".txt,.md,.pdf,.doc,.docx"
              multiple
              beforeUpload={() => false}
              onChange={(info) => {
                if (info.fileList.length > 0 && info.file.status === 'done') {
                  const files = info.fileList.map((f) => f.originFileObj!).filter(Boolean) as File[];
                  handleImport(files);
                }
              }}
            >
              <p className="text-gray-500">
                <ImportOutlined className="text-xl" />
              </p>
              <p className="text-gray-500 mt-2">点击或拖拽文件到此处上传</p>
              <p className="text-gray-400 text-sm mt-1">支持 txt, md, pdf, doc, docx 格式</p>
            </Upload.Dragger>

            {isUploading && (
              <div className="mt-4">
                <Progress percent={uploadProgress} />
                <p className="text-gray-500 text-sm mt-2">正在导入中...</p>
              </div>
            )}
          </Card>
        </Col>

        <Col span={12}>
          <Card
            title={
              <span className="flex items-center gap-2">
                <ExportOutlined />
                导出数据
              </span>
            }
          >
            <div className="space-y-4">
              <Button
                type="primary"
                icon={<FileTextOutlined />}
                className="w-full justify-start"
              >
                导出所有文档
              </Button>
              <Button
                type="primary"
                icon={<ForkOutlined />}
                className="w-full justify-start"
              >
                导出知识图谱
              </Button>
              <div className="text-gray-400 text-sm">
                导出格式为 JSON，包含完整的文档内容和图谱数据。
              </div>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default ImportExport;
