import { BellOutlined, UserOutlined, SearchOutlined } from '@ant-design/icons';
import { Button, Dropdown, Space } from 'antd';
import { Link, useNavigate } from 'react-router-dom';

const Header = () => {
  const navigate = useNavigate();

  const userMenu = [
    { key: '1', label: '个人设置' },
    { key: '2', label: '退出登录' },
  ];

  return (
    <header className="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-6 shadow-sm">
      <div className="flex items-center gap-4">
        <Link to="/" className="text-xl font-bold text-blue-600 flex items-center gap-2">
          <span className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center text-white">V</span>
          Vec-SVC
        </Link>
      </div>

      <div className="flex items-center gap-4">
        <Button
          icon={<SearchOutlined />}
          onClick={() => navigate('/search')}
          className="border-gray-200"
        >
          搜索
        </Button>

        <Space>
          <Button icon={<BellOutlined />} className="border-gray-200" />
          <Dropdown menu={{ items: userMenu }} placement="bottomRight">
            <Button icon={<UserOutlined />} className="border-gray-200">
              管理员
            </Button>
          </Dropdown>
        </Space>
      </div>
    </header>
  );
};

export default Header;
