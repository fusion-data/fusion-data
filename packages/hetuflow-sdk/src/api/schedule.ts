import { HetuflowClient } from '../utils/client.js';
import {
  ScheduleForCreate,
  ScheduleForUpdate,
  ScheduleForQuery,
  SchedSchedule,
  ScheduleStatus,
  ScheduleKind,
  PageResult_SchedSchedule,
  IdUuidResult,
} from '../types/index.js';

export class ScheduleAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 查询调度计划列表
   */
  async querySchedules(query: ScheduleForQuery): Promise<PageResult_SchedSchedule> {
    return this.client.post<PageResult_SchedSchedule>('/api/v1/schedules/page', query);
  }

  /**
   * 创建调度计划
   */
  async createSchedule(data: ScheduleForCreate): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>('/api/v1/schedules/item', data);
  }

  /**
   * 获取单个调度计划
   */
  async getSchedule(id: string): Promise<SchedSchedule | null> {
    return this.client.get<SchedSchedule | null>(`/api/v1/schedules/item/${id}`);
  }

  /**
   * 更新调度计划
   */
  async updateSchedule(id: string, data: ScheduleForUpdate): Promise<void> {
    return this.client.put<void>(`/api/v1/schedules/item/${id}`, data);
  }

  /**
   * 删除调度计划
   */
  async deleteSchedule(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/schedules/item/${id}`);
  }

  /**
   * 获取可调度的调度计划列表
   */
  async getSchedulableSchedules(): Promise<SchedSchedule[]> {
    return this.client.get<SchedSchedule[]>('/api/v1/schedules/schedulable');
  }

  /**
   * 启用调度计划
   */
  async enableSchedule(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/schedules/item/${id}/enable`);
  }

  /**
   * 禁用调度计划
   */
  async disableSchedule(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/schedules/item/${id}/disable`);
  }

  /**
   * 执行调度计划
   */
  async executeSchedule(id: string): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>(`/api/v1/schedules/item/${id}/execute`);
  }

  /**
   * 批量删除调度计划
   */
  async batchDeleteSchedules(ids: string[]): Promise<void> {
    await Promise.all(ids.map(id => this.deleteSchedule(id)));
  }

  /**
   * 批量启用调度计划
   */
  async batchEnableSchedules(ids: string[]): Promise<void> {
    await Promise.all(ids.map(id => this.enableSchedule(id)));
  }

  /**
   * 批量禁用调度计划
   */
  async batchDisableSchedules(ids: string[]): Promise<void> {
    await Promise.all(ids.map(id => this.disableSchedule(id)));
  }

  /**
   * 检查调度计划是否可执行
   */
  isScheduleExecutable(schedule: SchedSchedule): boolean {
    return schedule.status === ScheduleStatus.Enabled;
  }

  /**
   * 检查调度计划是否已过期
   */
  isScheduleExpired(schedule: SchedSchedule): boolean {
    return schedule.status === ScheduleStatus.Expired;
  }

  /**
   * 检查调度计划是否已禁用
   */
  isScheduleDisabled(schedule: SchedSchedule): boolean {
    return schedule.status === ScheduleStatus.Disabled;
  }

  /**
   * 获取调度计划状态文本
   */
  getScheduleStatusText(status: ScheduleStatus): string {
    const statusTextMap: Record<ScheduleStatus, string> = {
      [ScheduleStatus.Created]: '已创建',
      [ScheduleStatus.Expired]: '调度已过期',
      [ScheduleStatus.Disabled]: '已禁用',
      [ScheduleStatus.Enabled]: '已启用',
    };
    return statusTextMap[status] || '未知状态';
  }

  /**
   * 获取调度计划类型文本
   */
  getScheduleKindText(kind: ScheduleKind): string {
    const kindTextMap: Record<ScheduleKind, string> = {
      [ScheduleKind.Cron]: 'Cron 定时作业',
      [ScheduleKind.Interval]: '间隔定时作业',
      [ScheduleKind.Daemon]: '守护进程作业',
      [ScheduleKind.Event]: '事件驱动作业',
      [ScheduleKind.Flow]: '流程任务',
    };
    return kindTextMap[kind] || '未知类型';
  }

  /**
   * 获取调度计划状态选项
   */
  getScheduleStatusOptions(): Array<{ label: string; value: ScheduleStatus }> {
    return Object.entries({
      [ScheduleStatus.Created]: '已创建',
      [ScheduleStatus.Expired]: '调度已过期',
      [ScheduleStatus.Disabled]: '已禁用',
      [ScheduleStatus.Enabled]: '已启用',
    }).map(([value, label]) => ({
      label,
      value: parseInt(value) as ScheduleStatus,
    }));
  }

  /**
   * 获取调度计划类型选项
   */
  getScheduleKindOptions(): Array<{ label: string; value: ScheduleKind }> {
    return Object.entries({
      [ScheduleKind.Cron]: 'Cron 定时作业',
      [ScheduleKind.Interval]: '间隔定时作业',
      [ScheduleKind.Daemon]: '守护进程作业',
      [ScheduleKind.Event]: '事件驱动作业',
      [ScheduleKind.Flow]: '流程任务',
    }).map(([value, label]) => ({
      label,
      value: parseInt(value) as ScheduleKind,
    }));
  }

  /**
   * 获取调度计划的下一个执行时间
   */
  getNextRunTime(schedule: SchedSchedule): string | null {
    if (!this.isScheduleExecutable(schedule)) {
      return null;
    }
    return schedule.next_run_at || null;
  }

  /**
   * 验证调度计划配置
   */
  validateSchedule(schedule: Partial<ScheduleForCreate>): string[] {
    const errors: string[] = [];

    if (!schedule.id) {
      errors.push('调度计划ID不能为空');
    }

    if (!schedule.job_id) {
      errors.push('作业ID不能为空');
    }

    if (!schedule.schedule_kind) {
      errors.push('调度类型不能为空');
    }

    if (schedule.schedule_kind === ScheduleKind.Cron && !schedule.cron_expression) {
      errors.push('Cron调度类型需要提供cron表达式');
    }

    if (schedule.start_time && isNaN(Date.parse(schedule.start_time))) {
      errors.push('开始时间格式无效');
    }

    if (schedule.end_time && isNaN(Date.parse(schedule.end_time))) {
      errors.push('结束时间格式无效');
    }

    return errors;
  }

  /**
   * 获取调度计划摘要信息
   */
  getScheduleSummary(schedule: SchedSchedule): string {
    const kind = this.getScheduleKindText(schedule.schedule_kind);
    const status = this.getScheduleStatusText(schedule.status);

    let details = '';
    if (schedule.cron_expression) {
      details = ` (${schedule.cron_expression})`;
    } else if (schedule.interval_secs) {
      details = ` (${schedule.interval_secs}秒)`;
    }

    return `${kind} - ${status}${details}`;
  }

  /**
   * 根据状态筛选调度计划
   */
  filterSchedulesByStatus(schedules: SchedSchedule[], status: ScheduleStatus): SchedSchedule[] {
    return schedules.filter(schedule => schedule.status === status);
  }

  /**
   * 根据类型筛选调度计划
   */
  filterSchedulesByKind(schedules: SchedSchedule[], kind: ScheduleKind): SchedSchedule[] {
    return schedules.filter(schedule => schedule.schedule_kind === kind);
  }

  /**
   * 获取即将执行的调度计划
   */
  getUpcomingSchedules(schedules: SchedSchedule[]): SchedSchedule[] {
    const now = new Date();
    return schedules.filter(schedule => {
      if (!this.isScheduleExecutable(schedule) || !schedule.next_run_at) {
        return false;
      }
      const nextRunTime = new Date(schedule.next_run_at);
      return nextRunTime > now;
    });
  }
}
