import React, { useState, useEffect } from 'react';
import { Table, Card, Button, Space, Tag, Typography, Input, Row, Col, Tooltip, Popconfirm, Form } from 'antd';
import { ReloadOutlined, PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { apiService, SchedServer, ServerForQuery, ServerStatus } from '../../services/api';
import { useMessage } from '../../hooks/useMessage';
import dayjs from 'dayjs';

const { Title } = Typography;
const { Search } = Input;

// 可编辑单元格组件
interface EditableCellProps {
  editing: boolean;
  dataIndex: string;
  title: string;
  record: SchedServer;
  index: number;
  children: React.ReactNode;
  onSave: (id: string, field: string, value: any) => void;
}

const EditableCell: React.FC<EditableCellProps> = ({
  editing,
  dataIndex,
  title,
  record,
  index,
  children,
  onSave,
  ...restProps
}) => {
  const [form] = Form.useForm();
  const [isEditing, setIsEditing] = useState(false);
  const [_originalValue, setOriginalValue] = useState<any>(null);

  const save = async () => {
    try {
      const values = await form.validateFields();
      const value = values[dataIndex];

      let processedValue: any;
      let currentValue: any;

      // 处理绑定命名空间的特殊情况
      if (dataIndex === 'bind_namespaces') {
        processedValue = value
          .split(',')
          .map((ns: string) => ns.trim())
          .filter((ns: string) => ns);
        currentValue = (record.bind_namespaces as string[]) || [];
      } else {
        processedValue = value;
        currentValue = dataIndex === 'description' ? record.description || '' : '';
      }

      // 检查值是否真的有变化
      const hasChanged =
        dataIndex === 'bind_namespaces'
          ? JSON.stringify(processedValue.sort()) !== JSON.stringify(currentValue.sort())
          : processedValue !== currentValue;

      if (hasChanged) {
        onSave(record.id, dataIndex, processedValue);
      }

      setIsEditing(false);
      setOriginalValue(null);
    } catch (errInfo) {
      console.log('Save failed:', errInfo);
    }
  };

  const handleDoubleClick = () => {
    setIsEditing(true);
    const currentValue =
      dataIndex === 'bind_namespaces'
        ? (record.bind_namespaces as string[])?.join(', ') || ''
        : dataIndex === 'description'
          ? record.description || ''
          : '';

    // 存储原始值用于比较
    setOriginalValue(currentValue);

    form.setFieldsValue({
      [dataIndex]: currentValue,
    });
  };

  const handleBlur = () => {
    save();
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      save();
    }
    if (e.key === 'Escape') {
      setIsEditing(false);
    }
  };

  if (isEditing) {
    return (
      <td {...restProps}>
        <Form form={form} component={false}>
          <Form.Item
            name={dataIndex}
            style={{ margin: 0 }}
            rules={[
              {
                required: false,
              },
            ]}
          >
            <Input
              autoFocus
              onBlur={handleBlur}
              onKeyDown={handleKeyPress}
              placeholder={dataIndex === 'bind_namespaces' ? '多个命名空间用逗号分隔' : '请输入描述'}
            />
          </Form.Item>
        </Form>
      </td>
    );
  }

  return (
    <td {...restProps} onDoubleClick={handleDoubleClick} style={{ cursor: 'pointer' }}>
      {children}
    </td>
  );
};

interface ServersPageState {
  servers: SchedServer[];
  loading: boolean;
  total: number;
  current: number;
  pageSize: number;
  searchText: string;
}

/**
 * 服务器管理页面组件
 * 显示 SchedServer 列表和操作
 */
const Servers: React.FC = () => {
  const message = useMessage();
  const [state, setState] = useState<ServersPageState>({
    servers: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: '',
  });

  /**
   * 获取服务器列表
   */
  const fetchServers = async (params?: Partial<ServerForQuery>) => {
    try {
      setState(prev => ({ ...prev, loading: true }));

      const query: ServerForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.servers.queryServers(query);

      setState(prev => ({
        ...prev,
        servers: result.result || [],
        total: result.page.total || 0,
        current: query.page.page || 1,
        loading: false,
      }));
    } catch (error) {
      console.error('获取服务器列表失败:', error);
      message.error('获取服务器列表失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  // 组件挂载时获取数据
  useEffect(() => {
    fetchServers();
  }, []);

  // 状态标签渲染
  const renderStatus = (status: ServerStatus) => {
    const statusConfig = {
      [ServerStatus.Active]: { color: 'green', text: '活跃' },
      [ServerStatus.Inactive]: { color: 'red', text: '非活跃' },
    };
    const config = statusConfig[status];
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  // 表格列定义
  const columns: ColumnsType<SchedServer> = [
    {
      title: '服务器名称',
      dataIndex: 'name',
      key: 'name',
      sorter: (a, b) => a.name.localeCompare(b.name),
    },
    {
      title: '地址',
      dataIndex: 'address',
      key: 'address',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: renderStatus,
      filters: [
        { text: '活跃', value: ServerStatus.Active },
        { text: '非活跃', value: ServerStatus.Inactive },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: '最后心跳',
      dataIndex: 'last_heartbeat_at',
      key: 'last_heartbeat_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
      sorter: (a, b) => new Date(a.last_heartbeat_at).getTime() - new Date(b.last_heartbeat_at).getTime(),
    },
    {
      title: '绑定命名空间',
      dataIndex: 'bind_namespaces',
      key: 'bind_namespaces',
      onCell: (record: SchedServer) => ({
        record,
        dataIndex: 'bind_namespaces',
        title: '绑定命名空间',
        editing: false,
        onSave: handleCellUpdate,
      }),
      render: (namespaces: string[]) => (
        <Space wrap>
          {namespaces?.map(ns => <Tag key={ns}>{ns}</Tag>) || <span style={{ color: '#999' }}>双击编辑</span>}
        </Space>
      ),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
      onCell: (record: SchedServer) => ({
        record,
        dataIndex: 'description',
        title: '描述',
        editing: false,
        onSave: handleCellUpdate,
      }),
      render: (text: string) => text || <span style={{ color: '#999' }}>双击编辑</span>,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
      sorter: (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record) => (
        <Space size="middle">
          <Tooltip title="删除">
            <Popconfirm
              title="确认删除"
              description="确定要删除这个服务器吗？此操作不可撤销。"
              okText="确定"
              cancelText="取消"
              onConfirm={() => handleDelete(record.id)}
            >
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Popconfirm>
          </Tooltip>
        </Space>
      ),
    },
  ];

  // 处理搜索
  const handleSearch = (value: string) => {
    setState(prev => ({ ...prev, searchText: value, current: 1 }));
    fetchServers({
      page: { page: 1, limit: state.pageSize },
      filter: value ? { name: { like: `%${value}%` } } : {},
    });
  };

  // 处理刷新
  const handleRefresh = () => {
    fetchServers();
  };

  // 处理添加
  const handleAdd = () => {
    // TODO: 打开添加服务器对话框
    message.info('添加服务器功能开发中');
  };

  // 处理删除
  const handleDelete = async (id: string) => {
    try {
      await apiService.servers.deleteServer(id);
      message.success('删除成功');
      fetchServers();
    } catch (error) {
      console.error('删除服务器失败:', error);
      message.error('删除服务器失败');
    }
  };

  // 处理单元格更新
  const handleCellUpdate = async (id: string, field: string, value: any) => {
    try {
      const updateData: any = {};
      updateData[field] = value;

      await apiService.servers.updateServer(id, updateData);
      message.success('更新成功');

      // 更新本地状态
      setState(prev => ({
        ...prev,
        servers: prev.servers.map(server => (server.id === id ? { ...server, [field]: value } : server)),
      }));
    } catch (error) {
      console.error('更新服务器失败:', error);
      message.error('更新服务器失败');
    }
  };

  // 处理分页变化
  const handleTableChange = (pagination: any) => {
    const { current, pageSize } = pagination;
    setState(prev => ({ ...prev, current, pageSize }));
    fetchServers({
      page: { page: current, limit: pageSize },
      filter: state.searchText ? { name: { like: `%${state.searchText}%` } } : {},
    });
  };

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Row justify="space-between" align="middle">
        <Col>
          <Title level={2}>服务器管理</Title>
        </Col>
      </Row>

      <Card>
        <Row justify="space-between" style={{ marginBottom: 16 }}>
          <Col>
            <Space>
              <Search
                placeholder="搜索服务器名称"
                allowClear
                style={{ width: 300 }}
                onSearch={handleSearch}
                onChange={e => {
                  if (!e.target.value) {
                    setState(prev => ({ ...prev, searchText: '' }));
                    fetchServers();
                  }
                }}
              />
            </Space>
          </Col>
          <Col>
            <Space>
              <Button icon={<ReloadOutlined />} onClick={handleRefresh} loading={state.loading}>
                刷新
              </Button>
              <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
                添加服务器
              </Button>
            </Space>
          </Col>
        </Row>

        <Table
          columns={columns}
          dataSource={state.servers}
          rowKey="id"
          loading={state.loading}
          components={{
            body: {
              cell: EditableCell,
            },
          }}
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, size) => handleTableChange({ current: page, pageSize: size }),
          }}
        />
      </Card>
    </Space>
  );
};

export default Servers;
