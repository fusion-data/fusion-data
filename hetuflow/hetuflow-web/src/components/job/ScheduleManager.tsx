import React, { useState, useEffect } from 'react';
import { Form } from 'antd';
import { Card, Button, Space, Table, Popconfirm, Modal, Tooltip, Tag, message, Row, Col } from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  PlusOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import {
  ProForm,
  ProFormDependency,
  ProFormText,
  ProFormTextArea,
  ProFormSelect,
  ProFormDateTimePicker,
  ProFormDigit,
} from '@ant-design/pro-components';
import dayjs from 'dayjs';
import { v7 as uuidv7 } from 'uuid';
import { apiService, SchedSchedule, ScheduleKind, ScheduleStatus } from '@/services/api';

interface ScheduleManagerProps {
  jobId: string;
  readonly?: boolean;
}

interface ScheduleManagerState {
  schedules: SchedSchedule[];
  loading: boolean;
  scheduleModalVisible: boolean;
  editingSchedule: SchedSchedule | null;
  scheduleFormSubmitting: boolean;
}

const ScheduleManager: React.FC<ScheduleManagerProps> = ({ jobId, readonly = false }) => {
  const [scheduleForm] = Form.useForm();

  const [state, setState] = useState<ScheduleManagerState>({
    schedules: [],
    loading: false,
    scheduleModalVisible: false,
    editingSchedule: null,
    scheduleFormSubmitting: false,
  });

  const fetchSchedules = async () => {
    try {
      const response = await apiService.schedules.querySchedules({
        page: { page: 1, limit: 100 },
        filter: { job_id: { $eq: jobId } },
      });
      setState(prev => ({ ...prev, schedules: response.result || [] }));
    } catch (error) {
      console.error('获取调度计划失败:', error);
      message.error('获取调度计划失败');
    }
  };

  useEffect(() => {
    if (jobId) {
      fetchSchedules();
    }
  }, [jobId]);

  const handleCreateSchedule = () => {
    setState(prev => ({
      ...prev,
      scheduleModalVisible: true,
      editingSchedule: null,
    }));
    scheduleForm.resetFields();
  };

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
        await apiService.schedules.createSchedule({
          ...scheduleData,
          id: uuidv7(),
          job_id: jobId,
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

  const renderScheduleConfig = (record: SchedSchedule) => {
    if (record.schedule_kind === ScheduleKind.Cron) {
      return <span style={{ fontFamily: 'monospace' }}>{record.cron_expression}</span>;
    } else if (record.schedule_kind === ScheduleKind.Interval) {
      return (
        <Space>
          <span style={{ fontFamily: 'monospace' }}>{record.interval_secs}秒</span>
          {record.max_count && <span style={{ fontFamily: 'monospace' }}>最多{record.max_count}次</span>}
        </Space>
      );
    }
    return <span style={{ color: '#999' }}>-</span>;
  };

  const renderValidTime = (record: SchedSchedule) => {
    if (!record.start_time && !record.end_time) {
      return <span style={{ color: '#999' }}>-</span>;
    }

    const title_start = record.start_time ? dayjs(record.start_time).format('YYYY-MM-DDTHH:mmZ') : '永久';
    const title_end = record.end_time ? dayjs(record.end_time).format('YYYY-MM-DDTHH:mmZ') : '永久';
    const start = record.start_time ? dayjs(record.start_time).format('MM-DDTHH:mm') : '永久';
    const end = record.end_time ? dayjs(record.end_time).format('MM-DDTHH:mm') : '永久';

    return (
      <Tooltip
        title={
          <>
            开始: {title_start}
            <br />
            结束: {title_end}
          </>
        }
      >
        {start} ~ {end}
      </Tooltip>
    );
  };

  const scheduleColumns: ColumnsType<SchedSchedule> = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      render: (text, record) => <Tooltip title={record.id}>{text || record.id}</Tooltip>,
    },
    {
      title: '类型',
      dataIndex: 'schedule_kind',
      key: 'schedule_kind',
      width: 80,
      render: getScheduleKindTag,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 80,
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
      width: 120,
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
      width: 220,
      render: renderValidTime,
    },
    ...(readonly ? [] : [{
      title: '操作',
      key: 'action',
      width: 180,
      render: (_: any, record: SchedSchedule) => (
        <div>
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
        </div>
      ),
    }]),
  ];

  return (
    <Card
      title="调度计划"
      extra={!readonly && (
        <Button type="primary" icon={<PlusOutlined />} onClick={handleCreateSchedule}>
          创建调度计划
        </Button>
      )}
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
        <ProForm form={scheduleForm} layout="vertical" submitter={false}>
          <Row gutter={16}>
            <Col span={12}>
              <ProFormText name="name" label="名称" placeholder="请输入调度计划名称" />
            </Col>
            <Col span={12}>
              <ProFormSelect
                name="schedule_kind"
                label="调度类型"
                placeholder="请选择调度类型"
                rules={[{ required: true, message: '请选择调度类型' }]}
                options={[
                  { label: 'Cron 定时作业', value: ScheduleKind.Cron },
                  { label: '间隔定时作业', value: ScheduleKind.Interval },
                  { label: '守护进程作业', value: ScheduleKind.Daemon },
                  { label: '事件驱动作业', value: ScheduleKind.Event },
                  { label: '流程任务', value: ScheduleKind.Flow },
                ]}
              />
            </Col>
          </Row>

          {/* 动态表单字段 */}
          <ProFormDependency name={['schedule_kind']}>
            {(values: any) => {
              if (values.schedule_kind === ScheduleKind.Cron) {
                return (
                  <ProFormText
                    name="cron_expression"
                    label="Cron 表达式"
                    placeholder="例如: 0 0 12 * * ?"
                    rules={[{ required: true, message: '请输入 Cron 表达式' }]}
                  />
                );
              }

              if (values.schedule_kind === ScheduleKind.Interval) {
                return (
                  <Row gutter={16}>
                    <Col span={12}>
                      <ProFormDigit
                        name="interval_secs"
                        label="间隔秒数"
                        placeholder="请输入间隔秒数"
                        rules={[{ required: true, message: '请输入间隔秒数' }]}
                        fieldProps={{ min: 1 }}
                      />
                    </Col>
                    <Col span={12}>
                      <ProFormDigit
                        name="max_count"
                        label="最大执行次数"
                        placeholder="请输入最大执行次数"
                        rules={[{ required: true, message: '请输入最大执行次数' }]}
                        fieldProps={{ min: 1 }}
                      />
                    </Col>
                  </Row>
                );
              }

              return null;
            }}
          </ProFormDependency>
          <Row gutter={16}>
            <Col span={12}>
              <ProFormDateTimePicker name="start_time" label="开始时间" placeholder="选择开始时间" />
            </Col>
            <Col span={12}>
              <ProFormDateTimePicker name="end_time" label="结束时间" placeholder="选择结束时间" />
            </Col>
          </Row>
          <Row gutter={16}>
            <Col span={12}>
              <ProFormSelect
                name="status"
                label="状态"
                initialValue={ScheduleStatus.Enabled}
                options={[
                  { label: '已创建', value: ScheduleStatus.Created },
                  { label: '已过期', value: ScheduleStatus.Expired },
                  { label: '已禁用', value: ScheduleStatus.Disabled },
                  { label: '已启用', value: ScheduleStatus.Enabled },
                ]}
              />
            </Col>
          </Row>
          <ProFormTextArea name="description" label="描述" placeholder="请输入调度计划描述" fieldProps={{ rows: 3 }} />
        </ProForm>
      </Modal>
    </Card>
  );
};

export default ScheduleManager;