import { Layout } from 'antd';
import type { ReactNode } from 'react';
import Header from './Header';
import Sidebar from './Sidebar';

const { Content } = Layout;

interface AppLayoutProps {
  children?: ReactNode;
}

const AppLayout = ({ children }: AppLayoutProps) => {
  return (
    <Layout className="h-screen">
      <Sidebar />
      <Layout>
        <Header />
        <Content className="overflow-y-auto bg-gray-50 p-6">
          {children}
        </Content>
      </Layout>
    </Layout>
  );
};

export default AppLayout;
