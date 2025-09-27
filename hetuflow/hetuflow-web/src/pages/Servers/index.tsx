import React, { useState, useEffect } from "react";
import { Table, Card, Button, Space, Tag, Typography, Input, Row, Col, Tooltip, message, Modal } from "antd";
import { PlusOutlined, ReloadOutlined, EditOutlined, DeleteOutlined } from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import { apiService, SchedServer, ServerForQuery, ServerStatus } from "../../services/api";
import dayjs from "dayjs";

const { Title } = Typography;
const { Search } = Input;

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
  const [state, setState] = useState<ServersPageState>({
    servers: [],
    loading: false,
    total: 0,
    current: 1,
    pageSize: 10,
    searchText: "",
  });

  /**
   * 获取服务器列表
   */
  const fetchServers = async (params?: Partial<ServerForQuery>) => {
    try {
      setState((prev) => ({ ...prev, loading: true }));

      const query: ServerForQuery = {
        page: {
          page: params?.page?.page || state.current,
          limit: params?.page?.limit || state.pageSize,
        },
        filter: params?.filter || {},
      };

      const result = await apiService.servers.queryServers(query);

      setState((prev) => ({
        ...prev,
        servers: result.result || [],
        total: result.page.total || 0,
        current: query.page.page || 1,
        loading: false,
      }));
    } catch (error) {
      console.error("获取服务器列表失败:", error);
      message.error("获取服务器列表失败");
      setState((prev) => ({ ...prev, loading: false }));
    }
  };

  // 组件挂载时获取数据
  useEffect(() => {
    fetchServers();
  }, []);

  // 状态标签渲染
  const renderStatus = (status: ServerStatus) => {
    const statusConfig = {
      [ServerStatus.Active]: { color: "green", text: "活跃" },
      [ServerStatus.Inactive]: { color: "red", text: "非活跃" },
    };
    const config = statusConfig[status];
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  // 表格列定义
  const columns: ColumnsType<SchedServer> = [
    {
      title: "服务器名称",
      dataIndex: "name",
      key: "name",
      sorter: (a, b) => a.name.localeCompare(b.name),
    },
    {
      title: "地址",
      dataIndex: "address",
      key: "address",
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
      render: renderStatus,
      filters: [
        { text: "活跃", value: ServerStatus.Active },
        { text: "非活跃", value: ServerStatus.Inactive },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "最后心跳",
      dataIndex: "last_heartbeat_at",
      key: "last_heartbeat_at",
      render: (time: string) => dayjs(time).format("YYYY-MM-DD HH:mm:ss"),
      sorter: (a, b) => new Date(a.last_heartbeat_at).getTime() - new Date(b.last_heartbeat_at).getTime(),
    },
    {
      title: "绑定命名空间",
      dataIndex: "bind_namespaces",
      key: "bind_namespaces",
      render: (namespaces: string[]) => (
        <Space wrap>
          {namespaces.map((ns) => (
            <Tag key={ns}>{ns}</Tag>
          ))}
        </Space>
      ),
    },
    {
      title: "描述",
      dataIndex: "description",
      key: "description",
      ellipsis: true,
    },
    {
      title: "创建时间",
      dataIndex: "created_at",
      key: "created_at",
      render: (time: string) => dayjs(time).format("YYYY-MM-DD HH:mm:ss"),
      sorter: (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
    },
    {
      title: "操作",
      key: "action",
      render: (_, record) => (
        <Space size="middle">
          <Tooltip title="编辑">
            <Button type="text" icon={<EditOutlined />} onClick={() => handleEdit(record.id)} />
          </Tooltip>
          <Tooltip title="删除">
            <Button type="text" danger icon={<DeleteOutlined />} onClick={() => handleDelete(record.id)} />
          </Tooltip>
        </Space>
      ),
    },
  ];

  // 处理搜索
  const handleSearch = (value: string) => {
    setState((prev) => ({ ...prev, searchText: value, current: 1 }));
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
    message.info("添加服务器功能开发中");
  };

  // 处理编辑
  const handleEdit = async (id: string) => {
    try {
      const server = await apiService.servers.getServer(id);
      if (server) {
        // TODO: 打开编辑服务器对话框
        message.info(`编辑服务器: ${server.name}`);
      }
    } catch (error) {
      console.error("获取服务器详情失败:", error);
      message.error("获取服务器详情失败");
    }
  };

  // 处理删除
  const handleDelete = (id: string) => {
    Modal.confirm({
      title: "确认删除",
      content: "确定要删除这个服务器吗？此操作不可撤销。",
      okText: "确定",
      cancelText: "取消",
      onOk: async () => {
        try {
          await apiService.servers.deleteServer(id);
          message.success("删除成功");
          fetchServers();
        } catch (error) {
          console.error("删除服务器失败:", error);
          message.error("删除服务器失败");
        }
      },
    });
  };

  // 处理分页变化
  const handleTableChange = (pagination: any) => {
    const { current, pageSize } = pagination;
    setState((prev) => ({ ...prev, current, pageSize }));
    fetchServers({
      page: { page: current, limit: pageSize },
      filter: state.searchText ? { name: { like: `%${state.searchText}%` } } : {},
    });
  };

  return (
    <Space direction="vertical" size="large" style={{ width: "100%" }}>
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
                onChange={(e) => {
                  if (!e.target.value) {
                    setState((prev) => ({ ...prev, searchText: "" }));
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
