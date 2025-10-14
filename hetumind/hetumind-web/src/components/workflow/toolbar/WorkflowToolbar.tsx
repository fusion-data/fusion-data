import React, { useCallback, useState } from 'react';
import {
  Button,
  Space,
  Dropdown,
  Modal,
  Form,
  Input,
  Select,
  message,
  Divider,
  Upload,
  Badge
} from 'antd';
import {
  SaveOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  StopOutlined,
  UndoOutlined,
  RedoOutlined,
  ZoomInOutlined,
  ZoomOutOutlined,
  ExpandOutlined,
  CompressOutlined,
  DownloadOutlined,
  UploadOutlined,
  SettingOutlined,
  BugOutlined,
  CopyOutlined,
  DeleteOutlined,
  FolderOpenOutlined,
  CloudUploadOutlined
} from '@ant-design/icons';
import { useReactFlow } from '@xyflow/react';
import { useWorkflowStore } from '@/stores';
// import { NodeFactory } from '../nodes/NodeFactory'; // Not used yet

interface WorkflowToolbarProps {
  workflowId?: string;
  readOnly?: boolean;
  onExecute?: () => void;
  onStop?: () => void;
}

export const WorkflowToolbar: React.FC<WorkflowToolbarProps> = ({
  workflowId: _workflowId, // Not used yet
  readOnly = false,
  onExecute,
  onStop,
}) => {
  const {
    zoomIn,
    zoomOut,
    fitView,
  } = useReactFlow();

  const {
    currentWorkflow,
    undo,
    redo,
    history,
  } = useWorkflowStore();

  const [isExecuting, setIsExecuting] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [settingsVisible, setSettingsVisible] = useState(false);
  const [importVisible, setImportVisible] = useState(false);
  const [form] = Form.useForm();

  // 保存工作流
  const handleSave = useCallback(async () => {
    try {
      // TODO: Pass actual workflow data when available
      console.log('Save workflow placeholder - currentWorkflow:', currentWorkflow);
      message.success('工作流保存成功');
    } catch (error: any) {
      message.error('保存失败: ' + (error?.message || '未知错误'));
    }
  }, [currentWorkflow]);

  // 执行工作流
  const handleExecute = useCallback(async () => {
    if (isExecuting) return;

    try {
      setIsExecuting(true);
      onExecute?.();
      message.success('工作流执行开始');
    } catch (error: any) {
      message.error('执行失败: ' + (error?.message || '未知错误'));
      setIsExecuting(false);
    }
  }, [isExecuting, onExecute]);

  // 停止执行
  const handleStop = useCallback(async () => {
    try {
      setIsExecuting(false);
      setIsPaused(false);
      onStop?.();
      message.success('工作流已停止');
    } catch (error: any) {
      message.error('停止失败: ' + (error?.message || '未知错误'));
    }
  }, [onStop]);

  // 暂停执行
  const handlePause = useCallback(() => {
    setIsPaused(!isPaused);
    message.info(isPaused ? '工作流已恢复' : '工作流已暂停');
  }, [isPaused]);

  // 撤销操作
  const handleUndo = useCallback(() => {
    undo();
  }, [undo]);

  // 重做操作
  const handleRedo = useCallback(() => {
    redo();
  }, [redo]);

  // 导入工作流
  const handleImport = useCallback((_file: any) => {
    // TODO: Implement file import functionality
    console.log('Import functionality not yet implemented');
    message.info('导入功能开发中');
  }, []);

  // 导出工作流
  const handleExport = useCallback(() => {
    if (!currentWorkflow) return;

    const dataStr = JSON.stringify(currentWorkflow, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,'+ encodeURIComponent(dataStr);

    const exportFileDefaultName = `workflow-${currentWorkflow.name || 'unnamed'}-${Date.now()}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();

    message.success('工作流导出成功');
  }, [currentWorkflow]);

  // 更多操作菜单
  const moreMenuItems = [
    {
      key: 'import',
      label: '导入工作流',
      icon: <UploadOutlined />,
      onClick: () => setImportVisible(true),
    },
    {
      key: 'export',
      label: '导出工作流',
      icon: <DownloadOutlined />,
      onClick: handleExport,
    },
    {
      key: 'settings',
      label: '工作流设置',
      icon: <SettingOutlined />,
      onClick: () => setSettingsVisible(true),
    },
    {
      key: 'debug',
      label: '调试模式',
      icon: <BugOutlined />,
      onClick: () => message.info('调试模式功能开发中'),
    },
  ];

  // 缩放菜单
  const zoomMenuItems = [
    {
      key: 'fit',
      label: '适应画布',
      icon: <ExpandOutlined />,
      onClick: () => fitView(),
    },
    {
      key: 'zoom-in',
      label: '放大',
      icon: <ZoomInOutlined />,
      onClick: () => zoomIn(),
    },
    {
      key: 'zoom-out',
      label: '缩小',
      icon: <ZoomOutOutlined />,
      onClick: () => zoomOut(),
    },
    {
      key: 'zoom-reset',
      label: '重置缩放',
      icon: <CompressOutlined />,
      onClick: () => {
        // TODO: Implement viewport reset
        message.info('重置缩放功能开发中');
      },
    },
  ];

  return (
    <div className="workflow-toolbar">
      <Space size="small">
        {/* 文件操作 */}
        <Button
          type="text"
          icon={<FolderOpenOutlined />}
          onClick={() => message.info('打开工作流功能开发中')}
        >
          打开
        </Button>

        <Button
          type="text"
          icon={<SaveOutlined />}
          onClick={handleSave}
          disabled={readOnly}
        >
          保存
        </Button>

        <Dropdown
          menu={{ items: moreMenuItems }}
          trigger={['click']}
        >
          <Button type="text" icon={<SettingOutlined />}>
            更多
          </Button>
        </Dropdown>

        <Divider type="vertical" />

        {/* 编辑操作 */}
        <Button
          type="text"
          icon={<UndoOutlined />}
          onClick={handleUndo}
          disabled={!history.past.length || readOnly}
        >
          撤销
        </Button>

        <Button
          type="text"
          icon={<RedoOutlined />}
          onClick={handleRedo}
          disabled={!history.future.length || readOnly}
        >
          重做
        </Button>

        <Button
          type="text"
          icon={<CopyOutlined />}
          onClick={() => message.info('复制功能开发中')}
          disabled={readOnly}
        >
          复制
        </Button>

        <Button
          type="text"
          icon={<DeleteOutlined />}
          onClick={() => message.info('删除功能开发中')}
          disabled={readOnly}
        >
          删除
        </Button>

        <Divider type="vertical" />

        {/* 视图操作 */}
        <Dropdown
          menu={{ items: zoomMenuItems }}
          trigger={['click']}
        >
          <Button type="text" icon={<ExpandOutlined />}>
            视图
          </Button>
        </Dropdown>

        <Button
          type="text"
          icon={<BugOutlined />}
          onClick={() => message.info('调试功能开发中')}
        >
          调试
        </Button>

        <Divider type="vertical" />

        {/* 执行操作 */}
        {!readOnly && (
          <>
            {!isExecuting ? (
              <Button
                type="primary"
                icon={<PlayCircleOutlined />}
                onClick={handleExecute}
              >
                运行
              </Button>
            ) : (
              <Space>
                <Button
                  type="default"
                  icon={isPaused ? <PlayCircleOutlined /> : <PauseCircleOutlined />}
                  onClick={handlePause}
                >
                  {isPaused ? '恢复' : '暂停'}
                </Button>
                <Button
                  danger
                  icon={<StopOutlined />}
                  onClick={handleStop}
                >
                  停止
                </Button>
              </Space>
            )}
          </>
        )}

        {/* 执行状态指示器 */}
        {isExecuting && (
          <Badge
            status={isPaused ? 'warning' : 'processing'}
            text={isPaused ? '已暂停' : '执行中'}
          />
        )}
      </Space>

      {/* 设置对话框 */}
      <Modal
        title="工作流设置"
        open={settingsVisible}
        onCancel={() => setSettingsVisible(false)}
        footer={[
          <Button key="cancel" onClick={() => setSettingsVisible(false)}>
            取消
          </Button>,
          <Button key="save" type="primary" onClick={() => setSettingsVisible(false)}>
            保存
          </Button>,
        ]}
      >
        <Form form={form} layout="vertical">
          <Form.Item label="工作流名称" name="name">
            <Input placeholder="输入工作流名称" />
          </Form.Item>
          <Form.Item label="描述" name="description">
            <Input.TextArea rows={3} placeholder="输入工作流描述" />
          </Form.Item>
          <Form.Item label="执行超时(秒)" name="timeout">
            <Input type="number" placeholder="默认 60 秒" />
          </Form.Item>
          <Form.Item label="错误处理" name="errorHandling">
            <Select defaultValue="stop">
              <Select.Option value="stop">停止执行</Select.Option>
              <Select.Option value="continue">继续执行</Select.Option>
              <Select.Option value="retry">重试执行</Select.Option>
            </Select>
          </Form.Item>
        </Form>
      </Modal>

      {/* 导入对话框 */}
      <Modal
        title="导入工作流"
        open={importVisible}
        onCancel={() => setImportVisible(false)}
        footer={[
          <Button key="cancel" onClick={() => setImportVisible(false)}>
            取消
          </Button>,
        ]}
      >
        <Upload
          accept=".json"
          beforeUpload={(file) => {
            handleImport(file);
            return false; // 阻止自动上传
          }}
        >
          <Button icon={<CloudUploadOutlined />}>选择文件</Button>
        </Upload>
        <div style={{ marginTop: 8, color: '#666' }}>
          请选择 JSON 格式的工作流文件
        </div>
      </Modal>
    </div>
  );
};

export default WorkflowToolbar;