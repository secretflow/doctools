import { Input } from 'antd';
import { useState } from 'react';

const { useFullTextSearch } = await import('@secretflow/dumi-plugin-search/client');

export function SearchBar() {
  const [query, setQuery] = useState('');
  useFullTextSearch(query);
  return <Input value={query} onChange={(e) => setQuery(e.target.value)} />;
}
