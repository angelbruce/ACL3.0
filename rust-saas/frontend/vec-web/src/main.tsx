import { StrictMode, Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import './index.css';
import Routes from './routes';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 2,
    },
  },
});

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Suspense
        fallback={
          <div className="flex items-center justify-center h-screen">
            <div className="text-gray-500">加载中...</div>
          </div>
        }
      >
        <Routes />
      </Suspense>
    </QueryClientProvider>
  </StrictMode>,
);
