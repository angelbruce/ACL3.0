import { useState } from 'react';
import { Layout, Menu } from 'antd';
import {
  DashboardOutlined,
  SearchOutlined,
  FileTextOutlined,
  ForkOutlined,
  ShakeOutlined,
  TagsOutlined,
  IeOutlined,
  BarChartOutlined,
  HistoryOutlined,
  ExportOutlined,
} from '@ant-design/icons';
import { useLocation, useNavigate } from 'react-router-dom';

const { Sider } = Layout;

const menuItems = [
  { key: '/', icon: <DashboardOutlined />, label: '仪表盘' },
  { key: '/search', icon: <SearchOutlined />, label: '搜索' },
  { key: '/documents', icon: <FileTextOutlined />, label: '文档管理' },
  { key: '/graph', icon: <ForkOutlined />, label: '知识图谱' },
  { key: '/distillation', icon: <ShakeOutlined />, label: '知识蒸馏' },
  { key: '/taxonomy', icon: <TagsOutlined />, label: '知识分类' },
  { key: '/boundary', icon: <IeOutlined />, label: '知识边界' },
  { key: '/analytics', icon: <BarChartOutlined />, label: '分析' },
  { key: '/version', icon: <HistoryOutlined />, label: '版本管理' },
  { key: '/tasks', icon: <BarChartOutlined />, label: '任务管理' },
  { key: '/import-export', icon: <ExportOutlined />, label: '导入导出' },
];

const Sidebar = () => {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <Sider
      collapsible
      collapsed={collapsed}
      onCollapse={(value) => setCollapsed(value)}
      className="bg-white"
    >
      <div className="h-16 flex items-center justify-center border-b border-gray-100">
        <span className="text-lg font-bold text-blue-600">Vec-SVC</span>
      </div>
      <Menu
        mode="inline"
        selectedKeys={[location.pathname]}
        items={menuItems}
        onClick={({ key }) => navigate(key)}
      />
    </Sider>
  );
};

export default Sidebar;
