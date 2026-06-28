import { lazy, Suspense } from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import AppLayout from '@/components/Layout';

const Dashboard = lazy(() => import('@/pages/Dashboard'));
const Search = lazy(() => import('@/pages/Search'));
const DocumentList = lazy(() => import('@/pages/Documents/List'));
const DocumentDetail = lazy(() => import('@/pages/Documents/Detail'));
const KnowledgeGraph = lazy(() => import('@/pages/KnowledgeGraph'));
const Distillation = lazy(() => import('@/pages/Distillation'));
const Taxonomy = lazy(() => import('@/pages/Taxonomy'));
const Boundary = lazy(() => import('@/pages/Boundary'));
const Analytics = lazy(() => import('@/pages/Analytics'));
const Version = lazy(() => import('@/pages/Version'));
const Task = lazy(() => import('@/pages/Task'));
const ImportExport = lazy(() => import('@/pages/ImportExport'));

const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: 'search', element: <Search /> },
      { path: 'documents', element: <DocumentList /> },
      { path: 'documents/:id', element: <DocumentDetail /> },
      { path: 'graph', element: <KnowledgeGraph /> },
      { path: 'distillation', element: <Distillation /> },
      { path: 'taxonomy', element: <Taxonomy /> },
      { path: 'boundary', element: <Boundary /> },
      { path: 'analytics', element: <Analytics /> },
      { path: 'version', element: <Version /> },
      { path: 'tasks', element: <Task /> },
      { path: 'import-export', element: <ImportExport /> },
    ],
  },
]);

const Routes = () => {
  return (
    <Suspense
      fallback={
        <div className="flex items-center justify-center h-screen">
          <div className="text-gray-500">加载中...</div>
        </div>
      }
    >
      <RouterProvider router={router} />
    </Suspense>
  );
};

export default Routes;
