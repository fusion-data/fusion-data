import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Form } from 'antd';
import { Card, Button, Space, Typography, Row, Col, Tag, message } from 'antd';
import { ArrowLeftOutlined, SaveOutlined, ReloadOutlined } from '@ant-design/icons';
import { ProForm, ProFormText, ProFormTextArea, ProFormRadio } from '@ant-design/pro-components';
import dayjs from 'dayjs';
import { apiService, SchedJob } from '@/services/api';
import ScheduleManager from '@/components/job/ScheduleManager';

const { Title, Text } = Typography;

interface JobDetailState {
  job: SchedJob | null;
  loading: boolean;
  saving: boolean;
  disabled: boolean;
}

/**
 * 作业详情页面组件
 * 提供作业详细信息查看、编辑和调度计划管理功能
 */
const JobDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [form] = Form.useForm();

  const [state, setState] = useState<JobDetailState>({
    job: null,
    loading: false,
    saving: false,
    disabled: false,
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

  useEffect(() => {
    if (id) {
      fetchJobDetail();
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
        <ProForm
          form={form}
          layout="vertical"
          submitter={{
            render: () => (
              <Row style={{ marginTop: 16 }}>
                <Col>
                  <Space>
                    <Button onClick={() => navigate('/jobs')}>取消</Button>
                    <Button type="primary" icon={<SaveOutlined />} onClick={handleSaveJob} loading={state.saving}>
                      保存
                    </Button>
                  </Space>
                </Col>
              </Row>
            ),
          }}
        >
          <Row gutter={24}>
            <Col span={12}>
              <ProFormText
                name="name"
                label="作业名称"
                placeholder="请输入作业名称"
                rules={[{ required: true, message: '请输入作业名称' }]}
              />
            </Col>
            <Col span={12}>
              <ProFormRadio.Group
                name="status"
                label="状态"
                options={[
                  { label: <Tag color="red">禁用</Tag>, value: 99 },
                  { label: <Tag color="green">启用</Tag>, value: 100 },
                ]}
              />
            </Col>
          </Row>
          <Row>
            <Col span={24}>
              <ProFormTextArea name="description" label="描述" placeholder="请输入作业描述" fieldProps={{ rows: 3 }} />
            </Col>
          </Row>

          {state.job && (
            <Row gutter={24}>
              <Col span={8}>
                <Text type="secondary">作业ID：</Text>
                <Text code>{state.job.id}</Text>
              </Col>
              <Col span={8}>
                <Text type="secondary">创建时间：</Text>
                <Text>{dayjs(state.job.created_at).format('YYYY-MM-DD HH:mm:ss')}</Text>
              </Col>
              <Col span={8}>
                <Text type="secondary">更新时间：</Text>
                <Text>{dayjs(state.job.updated_at).format('YYYY-MM-DD HH:mm:ss')}</Text>
              </Col>
            </Row>
          )}
        </ProForm>
      </Card>

      {/* 调度计划管理组件 */}
      <ScheduleManager jobId={id || ''} />
    </Space>
  );
};

export default JobDetail;
