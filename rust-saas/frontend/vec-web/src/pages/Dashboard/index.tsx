import { Card, Row, Col, Statistic } from 'antd';
import { FileTextOutlined, SearchOutlined, ForkOutlined, BarChartOutlined } from '@ant-design/icons';
import { useQuery } from '@tanstack/react-query';
import { analyticsApi } from '@/api/analytics';

const Dashboard = () => {
  const { data: summary } = useQuery({
    queryKey: ['analytics-summary'],
    queryFn: analyticsApi.summary,
  });

  const stats = [
    {
      title: '文档总数',
      value: summary?.total_documents || 0,
      icon: <FileTextOutlined className="text-blue-500" />,
      suffix: '份',
    },
    {
      title: '搜索查询',
      value: summary?.total_search_queries || 0,
      icon: <SearchOutlined className="text-green-500" />,
      suffix: '次',
    },
    {
      title: '实体数量',
      value: summary?.total_entities || 0,
      icon: <ForkOutlined className="text-purple-500" />,
      suffix: '个',
    },
    {
      title: '访问次数',
      value: summary?.total_access_count || 0,
      icon: <BarChartOutlined className="text-orange-500" />,
      suffix: '次',
    },
  ];

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">仪表盘</h1>
      <Row gutter={16}>
        {stats.map((stat, index) => (
          <Col span={6} key={index}>
            <Card>
              <Statistic
                title={stat.title}
                value={stat.value}
                prefix={stat.icon}
                suffix={stat.suffix}
              />
            </Card>
          </Col>
        ))}
      </Row>
    </div>
  );
};

export default Dashboard;
