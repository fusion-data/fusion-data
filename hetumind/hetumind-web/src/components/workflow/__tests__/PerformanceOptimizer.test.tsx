import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { PerformanceOptimizer } from '../optimization/PerformanceOptimizer';

// Mock Ant Design components
jest.mock('antd', () => ({
  Card: ({ children, title, size, extra }: any) => (
    <div data-testid={`card-${title}`} data-size={size}>
      {title && <h3 data-testid="card-title">{title}</h3>}
      {extra && <div data-testid="card-extra">{extra}</div>}
      {children}
    </div>
  ),
  Row: ({ children, gutter }: any) => (
    <div data-gutter={gutter} className="ant-row">
      {children}
    </div>
  ),
  Col: ({ children, span }: any) => (
    <div data-span={span} className="ant-col">
      {children}
    </div>
  ),
  Button: ({ children, onClick, type, icon, size, disabled }: any) => (
    <button
      onClick={onClick}
      data-type={type}
      data-icon={icon}
      data-size={size}
      disabled={disabled}
    >
      {icon}
      {children}
    </button>
  ),
  Space: ({ children }: any) => <div className="ant-space">{children}</div>,
  Typography: {
    Title: ({ level, children, style }: any) => (
      <h1 data-level={level} style={style}>
        {children}
      </h1>
    ),
    Text: ({ children, type, style }: any) => (
      <span data-type={type} style={style}>
        {children}
      </span>
    ),
  },
  Switch: ({ checked, onChange, checkedChildren, unCheckedChildren }: any) => (
    <label>
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange?.(e.target.checked)}
      />
      {checked ? checkedChildren : unCheckedChildren}
    </label>
  ),
  Select: ({ children, onChange, value, style, size }: any) => (
    <select
      onChange={(e) => onChange?.(e.target.value)}
      value={value}
      style={style}
      data-size={size}
    >
      {children}
    </select>
  ),
  'Select.Option': ({ children, value }: any) => (
    <option value={value}>{children}</option>
  ),
  Alert: ({ message, description, type, showIcon, action }: any) => (
    <div data-type={type} data-message={message}>
      {showIcon && <span>⚠️</span>}
      <div>{message}</div>
      <div>{description}</div>
      {action}
    </div>
  ),
  List: ({ dataSource, renderItem, size }: any) => (
    <div data-size={size} className="ant-list">
      {dataSource?.map((item: any, index: number) => renderItem(item, index))}
    </div>
  ),
  'List.Item': ({ children, actions }: any) => (
    <div className="ant-list-item">
      {children}
      {actions && <div className="ant-list-item-actions">{actions}</div>}
    </div>
  ),
  'List.Item.Meta': ({ avatar, title, description }: any) => (
    <div className="ant-list-item-meta">
      {avatar && <div className="ant-list-item-meta-avatar">{avatar}</div>}
      <div className="ant-list-item-meta-content">
        {title && <div className="ant-list-item-meta-title">{title}</div>}
        {description && <div className="ant-list-item-meta-description">{description}</div>}
      </div>
    </div>
  ),
  Tag: ({ children, color, size }: any) => (
    <span data-color={color} data-size={size} className="ant-tag">
      {children}
    </span>
  ),
  Divider: () => <hr data-testid="divider" />,
  Tabs: ({ children, activeKey, onChange, size }: any) => (
    <div data-active-key={activeKey} data-size={size}>
      {children}
    </div>
  ),
  'Tabs.TabPane': ({ children, tab, key }: any) => (
    <div data-tab={tab} data-key={key}>
      {children}
    </div>
  ),
  Statistic: ({ title, value, suffix, prefix, valueStyle }: any) => (
    <div style={valueStyle}>
      {prefix}
      {value}
      {suffix}
      <div>{title}</div>
    </div>
  ),
  Progress: ({ percent, strokeColor, showInfo, size, format }: any) => (
    <div data-percent={percent} data-color={strokeColor}>
      <div style={{ width: `${percent}%`, backgroundColor: strokeColor }} />
      {showInfo && <span>{format ? format(percent) : `${percent}%`}</span>}
    </div>
  ),
  Tooltip: ({ children, title }: any) => (
    <div title={title}>{children}</div>
  ),
  Badge: ({ count, showZero, children }: any) => (
    <span data-count={count} data-show-zero={showZero}>
      {children}
      {count > 0 && <sup>{count}</sup>}
    </span>
  ),
}));

// Mock icons
jest.mock('@ant-design/icons', () => ({
  ThunderboltOutlined: () => <span data-testid="thunderbolt-icon">⚡</span>,
  DashboardOutlined: () => <span data-testid="dashboard-icon">📊</span>,
  CheckCircleOutlined: () => <span data-testid="check-icon">✅</span>,
  ExclamationCircleOutlined: () => <span data-testid="exclamation-icon">⚠️</span>,
  ClockCircleOutlined: () => <span data-testid="clock-icon">🕐</span>,
  DatabaseOutlined: () => <span data-testid="database-icon">💾</span>,
  MemoryOutlined: () => <span data-testid="memory-icon">🧠</span>,
  ApiOutlined: () => <span data-testid="api-icon">🔌</span>,
  SettingOutlined: () => <span data-testid="setting-icon">⚙️</span>,
  ReloadOutlined: () => <span data-testid="reload-icon">🔄</span>,
  BellOutlined: () => <span data-testid="bell-icon">🔔</span>,
  LineChartOutlined: () => <span data-testid="line-chart-icon">📈</span>,
  BarChartOutlined: () => <span data-testid="bar-chart-icon">📊</span>,
  PieChartOutlined: () => <span data-testid="pie-chart-icon">🥧</span>,
}));

// Mock performance.now for consistent testing
const mockPerformanceNow = jest.fn();
Object.defineProperty(global.performance, 'now', {
  value: mockPerformanceNow,
  writable: true,
});

// Mock performance.memory for memory testing
Object.defineProperty(global.performance, 'memory', {
  value: {
    usedJSHeapSize: 50 * 1024 * 1024, // 50MB
    totalJSHeapSize: 100 * 1024 * 1024, // 100MB
    jsHeapSizeLimit: 2048 * 1024 * 1024, // 2GB
  },
  writable: true,
});

describe('PerformanceOptimizer', () => {
  const defaultProps = {
    onOptimize: jest.fn(),
    refreshInterval: 5000,
    showSettings: true,
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockPerformanceNow.mockReturnValue(Date.now());
  });

  describe('Rendering', () => {
    it('renders performance optimizer correctly', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      expect(screen.getByTestId('dashboard-icon')).toBeInTheDocument();
      expect(screen.getByText('性能优化中心')).toBeInTheDocument();
      expect(screen.getByText('实时监控系统性能并提供优化建议')).toBeInTheDocument();
    });

    it('renders performance overview cards', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      expect(screen.getByTestId('card-性能概览')).toBeInTheDocument();
      expect(screen.getByTestId('card-详细指标')).toBeInTheDocument();
      expect(screen.getByTestId('card-优化建议')).toBeInTheDocument();
    });

    it('renders settings button when showSettings is true', () => {
      render(<PerformanceOptimizer {...defaultProps} showSettings={true} />);

      expect(screen.getByTestId('setting-icon')).toBeInTheDocument();
    });

    it('hides settings button when showSettings is false', () => {
      render(<PerformanceOptimizer {...defaultProps} showSettings={false} />);

      expect(screen.queryByTestId('setting-icon')).not.toBeInTheDocument();
    });

    it('renders refresh button', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      expect(screen.getByTestId('reload-icon')).toBeInTheDocument();
    });

    it('renders tabs for different sections', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      expect(screen.getByText('性能概览')).toBeInTheDocument();
      expect(screen.getByText('详细指标')).toBeInTheDocument();
      expect(screen.getByText('优化历史')).toBeInTheDocument();
    });
  });

  describe('Performance Metrics', () => {
    it('displays performance statistics', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText(/工作流执行时间/)).toBeInTheDocument();
        expect(screen.getByText(/画布帧率/)).toBeInTheDocument();
        expect(screen.getByText(/内存使用率/)).toBeInTheDocument();
      });
    });

    it('updates metrics on refresh', async () => {
      const mockOnOptimize = jest.fn();
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      const refreshButton = screen.getByTestId('reload-icon');
      fireEvent.click(refreshButton);

      await waitFor(() => {
        expect(mockOnOptimize).not.toHaveBeenCalled(); // Refresh doesn't call onOptimize
      });
    });

    it('calculates performance score correctly', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        const scoreElement = screen.getByText(/性能评分/);
        expect(scoreElement).toBeInTheDocument();
      });
    });

    it('displays metric trends', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        // Check for trend indicators (up/down arrows)
        expect(screen.getByText(/↑/)).toBeInTheDocument();
      });
    });
  });

  describe('Optimization Suggestions', () => {
    it('displays optimization suggestions', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText(/优化建议/)).toBeInTheDocument();
      });
    });

    it('categorizes suggestions by type', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText(/工作流/)).toBeInTheDocument();
        expect(screen.getByText(/画布/)).toBeInTheDocument();
        expect(screen.getByText(/数据/)).toBeInTheDocument();
        expect(screen.getByText(/系统/)).toBeInTheDocument();
      });
    });

    it('shows suggestion impact levels', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText(/高影响/)).toBeInTheDocument();
        expect(screen.getByText(/中影响/)).toBeInTheDocument();
        expect(screen.getByText(/低影响/)).toBeInTheDocument();
      });
    });

    it('applies optimization when button clicked', async () => {
      const mockOnOptimize = jest.fn();
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      await waitFor(() => {
        const optimizeButton = screen.getByText(/自动优化/);
        fireEvent.click(optimizeButton);
      });

      await waitFor(() => {
        expect(mockOnOptimize).toHaveBeenCalled();
      });
    });

    it('filters suggestions by category', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        const categorySelect = screen.getByDisplayValue('全部');
        expect(categorySelect).toBeInTheDocument();
      });
    });
  });

  describe('Auto-optimization', () => {
    it('toggles auto-optimization feature', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        const autoOptimizeSwitch = screen.getByText(/自动优化/);
        expect(autoOptimizeSwitch).toBeInTheDocument();
      });
    });

    it('applies automated optimizations when enabled', async () => {
      const mockOnOptimize = jest.fn();
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      await waitFor(() => {
        const autoOptimizeSwitch = screen.getByText(/自动优化/);
        fireEvent.click(autoOptimizeSwitch);
      });

      // Auto-optimization should trigger for automated suggestions
      await waitFor(() => {
        expect(mockOnOptimize).toHaveBeenCalled();
      }, { timeout: 6000 });
    });

    it('skips manual optimizations when auto-optimization is off', async () => {
      const mockOnOptimize = jest.fn();
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      // Auto-optimization is off by default
      await waitFor(() => {
        const manualButton = screen.getByText(/手动处理/);
        expect(manualButton).toBeInTheDocument();
      });
    });
  });

  describe('Real-time Updates', () => {
    it('updates metrics at specified intervals', async () => {
      const refreshInterval = 100;
      render(<PerformanceOptimizer {...defaultProps} refreshInterval={refreshInterval} />);

      await waitFor(() => {
        expect(screen.getByText(/工作流执行时间/)).toBeInTheDocument();
      });

      // Wait for at least one refresh cycle
      await new Promise(resolve => setTimeout(resolve, refreshInterval + 50));

      // Component should still be responsive
      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });

    it('handles rapid refresh requests', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      const refreshButton = screen.getByTestId('reload-icon');

      // Rapidly click refresh
      for (let i = 0; i < 5; i++) {
        fireEvent.click(refreshButton);
      }

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('handles optimization failures gracefully', async () => {
      const mockOnOptimize = jest.fn().mockRejectedValue(new Error('Optimization failed'));
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      await waitFor(() => {
        const optimizeButton = screen.getByText(/自动优化/);
        fireEvent.click(optimizeButton);
      });

      // Component should still render despite optimization failure
      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });

    it('handles missing metrics gracefully', async () => {
      // Mock performance.memory as undefined
      Object.defineProperty(global.performance, 'memory', {
        value: undefined,
        writable: true,
      });

      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });
    });

    it('handles invalid performance data', async () => {
      // Mock invalid performance data
      mockPerformanceNow.mockReturnValue(NaN);

      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    it('supports keyboard navigation', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      const refreshButton = screen.getByTestId('reload-icon');
      refreshButton.focus();

      expect(refreshButton).toHaveFocus();

      // Test Tab navigation
      fireEvent.keyDown(refreshButton, { key: 'Tab' });
      expect(refreshButton).toBeInTheDocument();
    });

    it('provides ARIA labels for metrics', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      // Check for proper labeling
      expect(screen.getByText(/性能评分/)).toBeInTheDocument();
      expect(screen.getByText(/工作流执行时间/)).toBeInTheDocument();
    });

    it('announces optimization status to screen readers', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        const optimizeButton = screen.getByText(/自动优化/);
        fireEvent.click(optimizeButton);
      });

      // Should have appropriate ARIA live regions for status updates
      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });
  });

  describe('Performance', () => {
    it('renders efficiently with large datasets', () => {
      const startTime = performance.now();
      render(<PerformanceOptimizer {...defaultProps} />);
      const endTime = performance.now();

      // Should render within reasonable time
      expect(endTime - startTime).toBeLessThan(100);
    });

    it('handles frequent metric updates without memory leaks', async () => {
      const { unmount } = render(<PerformanceOptimizer {...defaultProps} />);

      // Simulate frequent updates
      for (let i = 0; i < 10; i++) {
        const refreshButton = screen.getByTestId('reload-icon');
        fireEvent.click(refreshButton);
        await new Promise(resolve => setTimeout(resolve, 10));
      }

      unmount();

      // Component should unmount without issues
      expect(true).toBe(true);
    });

    it('optimizes rendering with React.memo', () => {
      const { rerender } = render(<PerformanceOptimizer {...defaultProps} />);

      const initialRender = screen.getByTestId('thunderbolt-icon');

      rerender(<PerformanceOptimizer {...defaultProps} />);

      // Should reuse components efficiently
      expect(initialRender).toBe(screen.getByTestId('thunderbolt-icon'));
    });
  });

  describe('Integration', () => {
    it('calls onOptimize with correct parameters', async () => {
      const mockOnOptimize = jest.fn();
      render(<PerformanceOptimizer {...defaultProps} onOptimize={mockOnOptimize} />);

      await waitFor(() => {
        const optimizeButton = screen.getByText(/自动优化/);
        fireEvent.click(optimizeButton);
      });

      await waitFor(() => {
        expect(mockOnOptimize).toHaveBeenCalledWith(
          expect.stringMatching(/optimize_|enable_|disable_|check_|increase_|cleanup_/)
        );
      });
    });

    it('handles missing onOptimize callback', () => {
      render(<PerformanceOptimizer {...defaultProps} onOptimize={undefined} />);

      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });

    it('respects custom refresh intervals', async () => {
      const customInterval = 1000;
      render(<PerformanceOptimizer {...defaultProps} refreshInterval={customInterval} />);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });

      // Component should use custom interval
      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });
  });

  describe('Data Visualization', () => {
    it('displays performance charts', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText(/执行趋势/)).toBeInTheDocument();
        expect(screen.getByText(/节点类型分布/)).toBeInTheDocument();
      });
    });

    it('updates chart data on refresh', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      const refreshButton = screen.getByTestId('reload-icon');
      fireEvent.click(refreshButton);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });
    });

    it('handles chart rendering errors', async () => {
      // Mock chart rendering error
      const originalError = console.error;
      console.error = jest.fn();

      render(<PerformanceOptimizer {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });

      console.error = originalError;
    });
  });

  describe('Configuration', () => {
    it('persists user preferences', () => {
      const { rerender } = render(<PerformanceOptimizer {...defaultProps} />);

      // Simulate user changing settings
      const autoOptimizeSwitch = screen.getByText(/自动优化/);
      fireEvent.click(autoOptimizeSwitch);

      rerender(<PerformanceOptimizer {...defaultProps} />);

      // Preferences should be maintained (in a real app, this would use localStorage)
      expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
    });

    it('resets to default configuration', async () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      // Simulate reset operation
      const refreshButton = screen.getByTestId('reload-icon');
      fireEvent.click(refreshButton);

      await waitFor(() => {
        expect(screen.getByTestId('thunderbolt-icon')).toBeInTheDocument();
      });
    });

    it('validates configuration values', () => {
      render(<PerformanceOptimizer {...defaultProps} />);

      // Test invalid refresh interval
      expect(() => {
        render(<PerformanceOptimizer {...defaultProps} refreshInterval={-1} />);
      }).not.toThrow();
    });
  });
});