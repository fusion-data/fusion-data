import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Card,
  Form,
  Input,
  Button,
  Space,
  Typography,
  Row,
  Col,
  Table,
  Switch,
  Select,
  DatePicker,
  Popconfirm,
  Modal,
  Tooltip,
  Tag,
  message,
} from 'antd';
import {
  ArrowLeftOutlined,
  EditOutlined,
  DeleteOutlined,
  PlusOutlined,
  SaveOutlined,
  ReloadOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { apiService, SchedJob, SchedSchedule, ScheduleKind, ScheduleStatus } from '../../services/api';

const { Title, Text } = Typography;
const { Option } = Select;
const { TextArea } = Input;

interface JobDetailState {
  job: SchedJob | null;
  schedules: SchedSchedule[];
  loading: boolean;
  editing: boolean;
  saving: boolean;
  scheduleModalVisible: boolean;
  editingSchedule: SchedSchedule | null;
  scheduleFormSubmitting: boolean;
}

/**
 * 作业详情页面组件
 * 提供作业详细信息查看、编辑和调度计划管理功能
 */
const JobDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [scheduleForm] = Form.useForm();

  const [state, setState] = useState<JobDetailState>({
    job: null,
    schedules: [],
    loading: false,
    editing: false,
    saving: false,
    scheduleModalVisible: false,
    editingSchedule: null,
    scheduleFormSubmitting: false,
  });

  /**
   * 获取作业详情
   */
  const fetchJobDetail = async () => {
    if (!id) return;

    try {
      setState(prev => ({ ...prev, loading: true }));
      const job = await apiService.jobs.getJob(id);
      setState(prev => ({
        ...prev,
        job,
        loading: false,
        editing: false,
      }));

      if (job) {
        form.setFieldsValue({
          name: job.name,
          description: job.description,
          status: job.status,
        });
      }
    } catch (error) {
      console.error('获取作业详情失败:', error);
      message.error('获取作业详情失败');
      setState(prev => ({ ...prev, loading: false }));
    }
  };

  /**
   * 获取作业调度计划
   */
  const fetchSchedules = async () => {
    if (!id) return;

    try {
      const response = await apiService.schedules.querySchedules({
        page: { page: 1, limit: 100 },
        filter: { job_id: { $eq: id } },
      });
      setState(prev => ({ ...prev, schedules: response.result || [] }));
    } catch (error) {
      console.error('获取调度计划失败:', error);
      message.error('获取调度计划失败');
    }
  };

  useEffect(() => {
    if (id) {
      fetchJobDetail();
      fetchSchedules();
    }
  }, [id]);

  /**
   * 保存作业信息
   */
  const handleSaveJob = async () => {
    try {
      const values = await form.validateFields();
      setState(prev => ({ ...prev, saving: true }));

      if (!id) {
        throw new Error('作业ID不能为空');
      }

      await apiService.jobs.updateJob(id, values);
      message.success('保存成功');
      setState(prev => ({ ...prev, saving: false, editing: false }));
      fetchJobDetail();
    } catch (error) {
      console.error('保存作业失败:', error);
      message.error('保存作业失败');
      setState(prev => ({ ...prev, saving: false }));
    }
  };

  /**
   * 切换作业状态
   */
  const handleToggleJobStatus = async (enabled: boolean) => {
    try {
      if (!id) {
        throw new Error('作业ID不能为空');
      }

      if (enabled) {
        await apiService.jobs.enableJob(id);
      } else {
        await apiService.jobs.disableJob(id);
      }
      message.success(enabled ? '启用成功' : '禁用成功');
      fetchJobDetail();
    } catch (error) {
      console.error('切换状态失败:', error);
      message.error('切换状态失败');
    }
  };

  /**
   * 打开创建调度计划弹窗
   */
  const handleCreateSchedule = () => {
    setState(prev => ({
      ...prev,
      scheduleModalVisible: true,
      editingSchedule: null,
    }));
    scheduleForm.resetFields();
  };

  /**
   * 打开编辑调度计划弹窗
   */
  const handleEditSchedule = (schedule: SchedSchedule) => {
    setState(prev => ({
      ...prev,
      scheduleModalVisible: true,
      editingSchedule: schedule,
    }));

    scheduleForm.setFieldsValue({
      name: schedule.name,
      description: schedule.description,
      schedule_kind: schedule.schedule_kind,
      cron_expression: schedule.cron_expression,
      interval_secs: schedule.interval_secs,
      max_count: schedule.max_count,
      start_time: schedule.start_time ? dayjs(schedule.start_time) : null,
      end_time: schedule.end_time ? dayjs(schedule.end_time) : null,
      status: schedule.status,
    });
  };

  /**
   * 删除调度计划
   */
  const handleDeleteSchedule = async (scheduleId: string) => {
    try {
      await apiService.schedules.deleteSchedule(scheduleId);
      message.success('删除成功');
      fetchSchedules();
    } catch (error) {
      console.error('删除调度计划失败:', error);
      message.error('删除调度计划失败');
    }
  };

  /**
   * 保存调度计划
   */
  const handleSaveSchedule = async () => {
    try {
      const values = await scheduleForm.validateFields();
      setState(prev => ({ ...prev, scheduleFormSubmitting: true }));

      const scheduleData = {
        ...values,
        start_time: values.start_time?.format(),
        end_time: values.end_time?.format(),
      };

      if (state.editingSchedule) {
        await apiService.schedules.updateSchedule(state.editingSchedule.id, scheduleData);
        message.success('更新成功');
      } else {
        if (!id) {
          throw new Error('作业ID不能为空');
        }
        await apiService.schedules.createSchedule({
          ...scheduleData,
          id: '',
          job_id: id,
        });
        message.success('创建成功');
      }

      setState(prev => ({
        ...prev,
        scheduleModalVisible: false,
        scheduleFormSubmitting: false,
      }));

      fetchSchedules();
    } catch (error) {
      console.error('保存调度计划失败:', error);
      message.error('保存调度计划失败');
      setState(prev => ({ ...prev, scheduleFormSubmitting: false }));
    }
  };

  /**
   * 获取调度计划类型标签
   */
  const getScheduleKindTag = (kind: ScheduleKind) => {
    const kindMap = {
      [ScheduleKind.Cron]: { color: 'blue', text: 'Cron' },
      [ScheduleKind.Interval]: { color: 'green', text: 'Interval' },
      [ScheduleKind.Daemon]: { color: 'purple', text: 'Daemon' },
      [ScheduleKind.Event]: { color: 'orange', text: 'Event' },
      [ScheduleKind.Flow]: { color: 'cyan', text: 'Flow' },
    };
    const config = kindMap[kind] || { color: 'default', text: 'Unknown' };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 获取状态标签
   */
  const getStatusTag = (status: ScheduleStatus) => {
    const statusMap = {
      [ScheduleStatus.Created]: { color: 'default', text: '已创建' },
      [ScheduleStatus.Expired]: { color: 'red', text: '已过期' },
      [ScheduleStatus.Disabled]: { color: 'orange', text: '已禁用' },
      [ScheduleStatus.Enabled]: { color: 'green', text: '已启用' },
    };
    const config = statusMap[status] || { color: 'default', text: 'Unknown' };
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  /**
   * 渲染调度配置信息
   */
  const renderScheduleConfig = (record: SchedSchedule) => {
    if (record.schedule_kind === ScheduleKind.Cron) {
      return <Text code>{record.cron_expression}</Text>;
    } else if (record.schedule_kind === ScheduleKind.Interval) {
      return (
        <Space>
          <Text code>{record.interval_secs}秒</Text>
          {record.max_count && <Text code>最多{record.max_count}次</Text>}
        </Space>
      );
    }
    return <Text type="secondary">-</Text>;
  };

  /**
   * 渲染有效时间
   */
  const renderValidTime = (record: SchedSchedule) => {
    if (!record.start_time && !record.end_time) {
      return <Text type="secondary">-</Text>;
    }

    const start = record.start_time ? dayjs(record.start_time).format('YYYY-MM-DD HH:mm') : '永久';
    const end = record.end_time ? dayjs(record.end_time).format('YYYY-MM-DD HH:mm') : '永久';

    return (
      <Tooltip title={`${start} - ${end}`}>
        <Text ellipsis style={{ maxWidth: 200 }}>
          {start} - {end}
        </Text>
      </Tooltip>
    );
  };

  /**
   * 调度计划表格列定义
   */
  const scheduleColumns: ColumnsType<SchedSchedule> = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      render: (text, record) => (
        <Space direction="vertical" size="small">
          <Text strong>{text || record.id}</Text>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {record.id}
          </Text>
        </Space>
      ),
    },
    {
      title: '类型',
      dataIndex: 'schedule_kind',
      key: 'schedule_kind',
      render: getScheduleKindTag,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      render: getStatusTag,
    },
    {
      title: '调度配置',
      key: 'config',
      render: renderScheduleConfig,
    },
    {
      title: '下次执行时间',
      dataIndex: 'next_run_at',
      key: 'next_run_at',
      render: (time: string) => (time ? dayjs(time).format('YYYY-MM-DD HH:mm:ss') : '-'),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
    },
    {
      title: '有效时间',
      key: 'valid_time',
      render: renderValidTime,
    },
    {
      title: '操作',
      key: 'action',
      width: 120,
      render: (_, record) => (
        <Space>
          <Button type="link" size="small" icon={<EditOutlined />} onClick={() => handleEditSchedule(record)}>
            编辑
          </Button>
          <Popconfirm
            title="确认删除"
            description="确定要删除这个调度计划吗？"
            icon={<ExclamationCircleOutlined style={{ color: 'red' }} />}
            okText="确定"
            cancelText="取消"
            onConfirm={() => handleDeleteSchedule(record.id)}
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      {/* 页面头部 */}
      <Row justify="space-between" align="middle">
        <Col>
          <Space size="middle">
            <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/jobs')}>
              返回
            </Button>
            <Title level={2} style={{ margin: 0 }}>
              作业详情
            </Title>
          </Space>
        </Col>
        <Col>
          <Space>
            <Button icon={<ReloadOutlined />} onClick={fetchJobDetail} loading={state.loading}>
              刷新
            </Button>
          </Space>
        </Col>
      </Row>

      {/* 作业信息卡片 */}
      <Card title="作业信息">
        <Form form={form} layout="vertical" disabled={!state.editing}>
          <Row gutter={24}>
            <Col span={12}>
              <Form.Item label="作业名称" name="name" rules={[{ required: true, message: '请输入作业名称' }]}>
                <Input placeholder="请输入作业名称" />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="状态" name="status">
                <Select disabled={!state.editing}>
                  <Option value={1}>已创建</Option>
                  <Option value={98}>已过期</Option>
                  <Option value={99}>已禁用</Option>
                  <Option value={100}>已启用</Option>
                </Select>
              </Form.Item>
            </Col>
          </Row>
          <Row>
            <Col span={24}>
              <Form.Item label="描述" name="description">
                <TextArea rows={3} placeholder="请输入作业描述" />
              </Form.Item>
            </Col>
          </Row>

          {!state.editing && state.job && (
            <Row gutter={24}>
              <Col span={8}>
                <Text type="secondary">创建时间：</Text>
                <Text>{dayjs(state.job.created_at).format('YYYY-MM-DD HH:mm:ss')}</Text>
              </Col>
              <Col span={8}>
                <Text type="secondary">更新时间：</Text>
                <Text>{dayjs(state.job.updated_at).format('YYYY-MM-DD HH:mm:ss')}</Text>
              </Col>
              <Col span={8}>
                <Text type="secondary">作业ID：</Text>
                <Text code>{state.job.id}</Text>
              </Col>
            </Row>
          )}

          <Row style={{ marginTop: 16 }}>
            <Col>
              <Space>
                {state.editing ? (
                  <>
                    <Button type="primary" icon={<SaveOutlined />} onClick={handleSaveJob} loading={state.saving}>
                      保存
                    </Button>
                    <Button onClick={() => setState(prev => ({ ...prev, editing: false }))}>取消</Button>
                  </>
                ) : (
                  <>
                    <Button
                      type="primary"
                      icon={<EditOutlined />}
                      onClick={() => setState(prev => ({ ...prev, editing: true }))}
                    >
                      编辑
                    </Button>
                    {state.job && (
                      <Switch
                        checked={state.job.status === 100}
                        checkedChildren="启用"
                        unCheckedChildren="禁用"
                        onChange={handleToggleJobStatus}
                      />
                    )}
                  </>
                )}
              </Space>
            </Col>
          </Row>
        </Form>
      </Card>

      {/* 调度计划卡片 */}
      <Card
        title="调度计划"
        extra={
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreateSchedule}>
            创建调度计划
          </Button>
        }
      >
        <Table
          columns={scheduleColumns}
          dataSource={state.schedules}
          rowKey="id"
          pagination={{
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
          }}
        />
      </Card>

      {/* 调度计划弹窗 */}
      <Modal
        title={state.editingSchedule ? '编辑调度计划' : '创建调度计划'}
        open={state.scheduleModalVisible}
        onCancel={() => setState(prev => ({ ...prev, scheduleModalVisible: false }))}
        footer={[
          <Button key="cancel" onClick={() => setState(prev => ({ ...prev, scheduleModalVisible: false }))}>
            取消
          </Button>,
          <Button key="submit" type="primary" loading={state.scheduleFormSubmitting} onClick={handleSaveSchedule}>
            保存
          </Button>,
        ]}
        width={600}
      >
        <Form form={scheduleForm} layout="vertical">
          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="名称" name="name">
                <Input placeholder="请输入调度计划名称" />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="调度类型" name="schedule_kind" rules={[{ required: true, message: '请选择调度类型' }]}>
                <Select placeholder="请选择调度类型">
                  <Option value={ScheduleKind.Cron}>Cron 定时作业</Option>
                  <Option value={ScheduleKind.Interval}>间隔定时作业</Option>
                  <Option value={ScheduleKind.Daemon}>守护进程作业</Option>
                  <Option value={ScheduleKind.Event}>事件驱动作业</Option>
                  <Option value={ScheduleKind.Flow}>流程任务</Option>
                </Select>
              </Form.Item>
            </Col>
          </Row>

          {/* 动态表单字段 */}
          <Form.Item noStyle shouldUpdate>
            {({ getFieldValue }) => {
              const scheduleKind = getFieldValue('schedule_kind');

              return (
                <>
                  {scheduleKind === ScheduleKind.Cron && (
                    <Form.Item
                      label="Cron 表达式"
                      name="cron_expression"
                      rules={[{ required: true, message: '请输入 Cron 表达式' }]}
                    >
                      <Input placeholder="例如: 0 0 12 * * ?" />
                    </Form.Item>
                  )}

                  {scheduleKind === ScheduleKind.Interval && (
                    <>
                      <Row gutter={16}>
                        <Col span={12}>
                          <Form.Item
                            label="间隔秒数"
                            name="interval_secs"
                            rules={[{ required: true, message: '请输入间隔秒数' }]}
                          >
                            <Input type="number" placeholder="请输入间隔秒数" min={1} />
                          </Form.Item>
                        </Col>
                        <Col span={12}>
                          <Form.Item
                            label="最大执行次数"
                            name="max_count"
                            rules={[{ required: true, message: '请输入最大执行次数' }]}
                          >
                            <Input type="number" placeholder="请输入最大执行次数" min={1} />
                          </Form.Item>
                        </Col>
                      </Row>
                    </>
                  )}
                </>
              );
            }}
          </Form.Item>

          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="开始时间" name="start_time">
                <DatePicker showTime placeholder="选择开始时间" style={{ width: '100%' }} />
              </Form.Item>
            </Col>
            <Col span={12}>
              <Form.Item label="结束时间" name="end_time">
                <DatePicker showTime placeholder="选择结束时间" style={{ width: '100%' }} />
              </Form.Item>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col span={12}>
              <Form.Item label="状态" name="status" initialValue={ScheduleStatus.Enabled}>
                <Select>
                  <Option value={ScheduleStatus.Created}>已创建</Option>
                  <Option value={ScheduleStatus.Expired}>已过期</Option>
                  <Option value={ScheduleStatus.Disabled}>已禁用</Option>
                  <Option value={ScheduleStatus.Enabled}>已启用</Option>
                </Select>
              </Form.Item>
            </Col>
          </Row>

          <Form.Item label="描述" name="description">
            <TextArea rows={3} placeholder="请输入调度计划描述" />
          </Form.Item>
        </Form>
      </Modal>
    </Space>
  );
};

export default JobDetail;
