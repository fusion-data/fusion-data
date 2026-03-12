---
name: web-frontend
description: React 19 + TypeScript frontend patterns for Fusion-Data, Ant Design, workflow canvas, API patterns
---

# Web Frontend Development

## Tech Stack

- React 19 + TypeScript 5.x
- Ant Design v5 + @ant-design/v5-patch-for-react-19
- Vite 7.x, pnpm workspaces
- Zustand (client state), React Query (server state)
- @xyflow/react (workflow canvas)

## Quick Patterns

### Component

```tsx
import { Button } from 'antd';
import type { ButtonProps } from 'antd';

interface Props extends Omit<ButtonProps, 'onClick'> {
  onClick: (id: string) => void;
}

export const Component: React.FC<Props> = ({ onClick, children, ...props }) => {
  return <Button onClick={() => onClick('id')}>{children}</Button>;
};
```

### API Service

```tsx
import axios from 'axios';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL,
  timeout: 10000,
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

export const service = {
  get: <T>(url: string) => api.get<T>(url).then(r => r.data),
  post: <T>(url: string, data: unknown) => api.post<T>(url, data).then(r => r.data),
};
```

### React Query

```tsx
import { useQuery } from '@tanstack/react-query';

export const useWorkflows = () => useQuery({
  queryKey: ['workflows'],
  queryFn: () => service.get('/api/workflows'),
  refetchInterval: 5000,
});
```

### Workflow Canvas

```tsx
import { ReactFlow, Background, Controls } from '@xyflow/react';
import '@xyflow/react/dist/style.css';

export const Canvas: React.FC<{ nodes: Node[]; edges: Edge[] }> = ({ nodes, edges }) => (
  <div style={{ height: 600 }}>
    <ReactFlow nodes={nodes} edges={edges} fitView>
      <Controls />
      <Background />
    </ReactFlow>
  </div>
);
```

## Build Commands

```bash
pnpm dev              # dev server
pnpm build            # production build
pnpm type-check       # TS check only
```

## Related Skills

- `rust-backend`: API integration patterns
