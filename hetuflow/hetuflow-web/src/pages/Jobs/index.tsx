import React, { useState, useEffect } from 'react';
import { Table, Card, Button, Space, Typography, Input, Row, Col, Switch, Dropdown, Popconfirm } from 'antd';
import {
  ReloadOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  PlayCircleOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { apiService, SchedJob, JobForQuery, JobStatus } from '../../services/api';
import { useMessage } from '../../hooks/useMessage';
import dayjs from 'dayjs';

const { Title } = Typography;
const { Search } = Input;

interface JobsPageState {
  jobs: SchedJob[];
  loading: boolean;
  total: number;
  current: number;
  pageSize: number;
  searchText: string;
}

/**
 * 作业管理页面组件
 * 提供作业的 CRUD 操作和状态管理
 */
const Jobs: React.FC = () => {
  const message = useMessage();
  const [state, setState] = useState<JobsPageState>({
    jobs: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: '',
  });
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  /**
   * 获取作业列表
   */
  const fetchJobs = async (params?: Partial<JobForQuery>) => {
    try {
      setState(prev => ({ ...prev, loading: true }));
      const query: JobForQuery = {
        page: params?.page || { page: state.current, limit: state.pageSize },
        filter: params?.filter || {
          name: state.searchText ? { $like: state.searchText } : undefined,
        },
      };

      const result = await apiService.jobs.queryJobs(query);

      setState(prev => ({
        ...prev,
        jobs: result.result || [],
        total: result.page.total || 0,
        current: query.page?.page || 1,
        loading: false,
      }));
    } catch (error) {
      console.error('获取作业列表失败:', error);
      message.error('获取作业列表失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  useEffect(() => {
    fetchJobs();
  }, []);

  /**
   * 表格列定义
   */
  const columns: ColumnsType<SchedJob> = [
    {
      title: '作业名称',
      dataIndex: 'name',
      key: 'name',
      sorter: (a, b) => a.name.localeCompare(b.name),
      render: (text, record) => (
        <Button type="link" onClick={() => handleEdit(record)} style={{ padding: 0, height: 'auto' }}>
          {text}
        </Button>
      ),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: (status, record) => (
        <Space>
          <Switch
            size="small"
            checked={status === JobStatus.ENABLED}
            onChange={checked => handleStatusChange(checked, record)}
          />
        </Space>
      ),
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
    },
    {
      title: '更新时间',
      dataIndex: 'updated_at',
      key: 'updated_at',
      render: (time: string) => dayjs(time).format('YYYY-MM-DD HH:mm:ss'),
    },
    {
      title: '操作',
      key: 'action',
      render: (_, record) => {
        const menuItems = [
          {
            key: 'edit',
            icon: <EditOutlined />,
            label: '编辑',
            onClick: () => handleEdit(record),
          },
          {
            key: 'run',
            icon: <PlayCircleOutlined />,
            label: '立即执行',
            onClick: () => handleExecute(record),
          },
          {
            type: 'divider' as const,
          },
          {
            key: 'delete',
            icon: (
              <Popconfirm
                title="确认删除"
                description={`确定要删除作业 "${record.name}" 吗？`}
                okText="确定"
                cancelText="取消"
                onConfirm={() => handleDelete(record)}
              >
                <DeleteOutlined />
              </Popconfirm>
            ),
            label: '删除',
            danger: true,
          },
        ];

        return (
          <Dropdown menu={{ items: menuItems }} trigger={['click']}>
            <Button type="text" icon={<MoreOutlined />} />
          </Dropdown>
        );
      },
    },
  ];

  /**
   * 处理搜索
   */
  const handleSearch = (value: string) => {
    setState(prev => ({ ...prev, searchText: value, current: 1 }));
    fetchJobs({
      page: { page: 1, limit: state.pageSize },
      filter: { name: value ? { $like: value } : undefined },
    });
  };

  /**
   * 刷新数据
   */
  const handleRefresh = () => {
    fetchJobs();
  };

  /**
   * 添加作业
   */
  const handleAdd = () => {
    // TODO: 打开添加作业对话框
    message.info('添加作业功能开发中');
  };

  /**
   * 编辑作业
   */
  const handleEdit = (record: SchedJob) => {
    // TODO: 打开编辑作业对话框
    message.info(`编辑作业 "${record.name}" 功能开发中`);
  };

  /**
   * 删除作业
   */
  const handleDelete = async (record: SchedJob) => {
    try {
      await apiService.jobs.deleteJob(record.id);
      message.success('删除成功');
      fetchJobs();
    } catch (error) {
      console.error('删除作业失败:', error);
      message.error('删除作业失败');
    }
  };

  /**
   * 立即执行
   */
  const handleExecute = (record: SchedJob) => {
    // TODO: 调用立即执行 API
    message.info(`立即执行作业 "${record.name}" 功能开发中`);
  };

  /**
   * 切换状态
   */
  const handleStatusChange = async (checked: boolean, record: SchedJob) => {
    try {
      if (checked) {
        await apiService.jobs.enableJob(record.id);
        message.success('启用成功');
      } else {
        await apiService.jobs.disableJob(record.id);
        message.success('禁用成功');
      }
      fetchJobs();
    } catch (error) {
      console.error('切换状态失败:', error);
      message.error('切换状态失败');
    }
  };

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Row justify="space-between" align="middle">
        <Col>
          <Title level={2}>作业管理</Title>
        </Col>
      </Row>

      <Card>
        <Row justify="space-between" style={{ marginBottom: 16 }}>
          <Col>
            <Space>
              <Search
                placeholder="搜索作业名称、描述或代理"
                allowClear
                style={{ width: 300 }}
                onSearch={handleSearch}
                onChange={e => setState(prev => ({ ...prev, searchText: e.target.value }))}
              />
            </Space>
          </Col>
          <Col>
            <Space>
              <Button icon={<ReloadOutlined />} onClick={handleRefresh} loading={state.loading}>
                刷新
              </Button>
              <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
                创建作业
              </Button>
            </Space>
          </Col>
        </Row>

        <Table
          columns={columns}
          dataSource={state.jobs}
          rowKey="id"
          loading={state.loading}
          rowSelection={{
            selectedRowKeys,
            onChange: setSelectedRowKeys,
          }}
          pagination={{
            current: state.current,
            pageSize: state.pageSize,
            total: state.total,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
            onChange: (page, pageSize) => {
              setState(prev => ({
                ...prev,
                current: page,
                pageSize: pageSize || 10,
              }));
              fetchJobs({ page: { page, limit: pageSize || 10 } });
            },
          }}
        />
      </Card>
    </Space>
  );
};

export default Jobs;
