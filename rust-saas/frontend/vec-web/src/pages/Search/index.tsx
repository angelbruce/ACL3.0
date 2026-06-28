import { useState } from 'react';
import { Input, Card, List, Tag, Button } from 'antd';
import { SearchOutlined, FileTextOutlined } from '@ant-design/icons';
import { useQuery } from '@tanstack/react-query';
import { searchApi } from '@/api/search';
import type { SearchResult } from '@/types/search';

const SearchPage = () => {
  const [query, setQuery] = useState('');
  const [searchKey, setSearchKey] = useState('');

  const { data: results } = useQuery({
    queryKey: ['search', searchKey],
    queryFn: () => searchApi.query(searchKey, { limit: 20 }),
    enabled: !!searchKey,
  });

  const handleSearch = () => {
    if (query.trim()) {
      setSearchKey(query);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">搜索</h1>
      <Card className="mb-6">
        <Input.Search
          placeholder="输入搜索关键词..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onSearch={handleSearch}
          onPressEnter={handleKeyPress}
          size="large"
          prefix={<SearchOutlined />}
          enterButton={<Button type="primary">搜索</Button>}
        />
      </Card>

      {searchKey && (
        <Card title={`搜索结果: "${searchKey}"`}>
          {results && results.length > 0 ? (
            <List
              dataSource={results}
              renderItem={(item: SearchResult) => (
                <List.Item
                  actions={[
                    <Tag color="blue">{(item.score * 100).toFixed(1)}%</Tag>,
                  ]}
                >
                  <List.Item.Meta
                    avatar={<FileTextOutlined className="text-gray-400" />}
                    title={<a href={`/documents/${item.document_id}`}>{item.document_title}</a>}
                    description={
                      <div
                        dangerouslySetInnerHTML={{ __html: item.highlighted_content || item.content }}
                      />
                    }
                  />
                </List.Item>
              )}
            />
          ) : (
            <div className="text-center py-10 text-gray-500">
              暂无搜索结果
            </div>
          )}
        </Card>
      )}
    </div>
  );
};

export default SearchPage;
