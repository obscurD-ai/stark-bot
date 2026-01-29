import { useState, useEffect, useCallback } from 'react';
import { ChevronDown, ChevronRight, DollarSign, Cpu, Clock } from 'lucide-react';
import clsx from 'clsx';
import { useGateway } from '@/hooks/useGateway';
import type { ExecutionEvent, X402PaymentEvent } from '@/types';

interface DebugTask {
  id: string;
  parentId?: string;
  name: string;
  description?: string;
  activeForm?: string;
  taskType?: string;
  status: 'pending' | 'in_progress' | 'completed' | 'error';
  startTime?: number;
  endTime?: number;
  duration?: number;
  toolsCount?: number;
  tokensUsed?: number;
  children: DebugTask[];
}

interface DebugPanelProps {
  className?: string;
}

export default function DebugPanel({ className }: DebugPanelProps) {
  const [executions, setExecutions] = useState<Map<string, DebugTask>>(new Map());
  const [payments, setPayments] = useState<X402PaymentEvent[]>([]);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState<'tasks' | 'payments'>('tasks');
  const { on, off } = useGateway();

  const updateExecution = useCallback((executionId: string, updater: (task: DebugTask) => DebugTask) => {
    setExecutions((prev) => {
      const newMap = new Map(prev);
      const execution = newMap.get(executionId);
      if (execution) {
        newMap.set(executionId, updater(execution));
      }
      return newMap;
    });
  }, []);

  const handleExecutionStarted = useCallback((data: unknown) => {
    const event = data as ExecutionEvent;

    const newExecution: DebugTask = {
      id: event.execution_id,
      name: 'Processing',
      description: `Execution started (mode: ${(data as Record<string, unknown>).mode || 'execute'})`,
      status: 'in_progress',
      startTime: Date.now(),
      children: [],
    };

    setExecutions((prev) => {
      const newMap = new Map(prev);
      newMap.set(event.execution_id, newExecution);
      return newMap;
    });
  }, []);

  const handleExecutionThinking = useCallback((data: unknown) => {
    const event = data as ExecutionEvent;
    updateExecution(event.execution_id, (execution) => ({
      ...execution,
      activeForm: event.active_form || (data as Record<string, unknown>).text as string || 'Thinking...',
    }));
  }, [updateExecution]);

  const handleTaskStarted = useCallback((data: unknown) => {
    const event = data as ExecutionEvent & {
      id?: string;
      type?: string;
      description?: string;
    };

    const newTask: DebugTask = {
      id: event.id || event.task_id || crypto.randomUUID(),
      parentId: event.parent_task_id || (data as Record<string, unknown>).parent_id as string,
      name: event.name || (data as Record<string, unknown>).description as string || 'Task',
      description: event.description || event.name,
      taskType: event.type || (data as Record<string, unknown>).type as string,
      activeForm: event.active_form,
      status: 'in_progress',
      startTime: Date.now(),
      children: [],
    };

    updateExecution(event.execution_id, (execution) => {
      const addToParent = (tasks: DebugTask[]): DebugTask[] => {
        return tasks.map((task) => {
          if (task.id === newTask.parentId) {
            return { ...task, children: [...task.children, newTask] };
          }
          return { ...task, children: addToParent(task.children) };
        });
      };

      if (!newTask.parentId || newTask.parentId === execution.id) {
        return { ...execution, children: [...execution.children, newTask] };
      }
      return { ...execution, children: addToParent(execution.children) };
    });
  }, [updateExecution]);

  const handleTaskUpdated = useCallback((data: unknown) => {
    const event = data as ExecutionEvent;
    if (!event.task_id) return;

    updateExecution(event.execution_id, (execution) => {
      const updateTask = (tasks: DebugTask[]): DebugTask[] => {
        return tasks.map((task) => {
          if (task.id === event.task_id) {
            return {
              ...task,
              toolsCount: event.tools_count ?? task.toolsCount,
              tokensUsed: event.tokens_used ?? task.tokensUsed,
              activeForm: event.active_form ?? task.activeForm,
            };
          }
          return { ...task, children: updateTask(task.children) };
        });
      };

      if (execution.id === event.task_id) {
        return {
          ...execution,
          toolsCount: event.tools_count ?? execution.toolsCount,
          tokensUsed: event.tokens_used ?? execution.tokensUsed,
          activeForm: event.active_form ?? execution.activeForm,
        };
      }
      return { ...execution, children: updateTask(execution.children) };
    });
  }, [updateExecution]);

  const handleTaskCompleted = useCallback((data: unknown) => {
    const event = data as ExecutionEvent;
    if (!event.task_id) return;

    updateExecution(event.execution_id, (execution) => {
      const completeTask = (tasks: DebugTask[]): DebugTask[] => {
        return tasks.map((task) => {
          if (task.id === event.task_id) {
            return {
              ...task,
              status: 'completed',
              endTime: Date.now(),
              duration: event.duration_ms ?? (Date.now() - (task.startTime || Date.now())),
            };
          }
          return { ...task, children: completeTask(task.children) };
        });
      };

      return { ...execution, children: completeTask(execution.children) };
    });
  }, [updateExecution]);

  const handleExecutionCompleted = useCallback((data: unknown) => {
    const event = data as ExecutionEvent;

    updateExecution(event.execution_id, (execution) => ({
      ...execution,
      status: 'completed',
      endTime: Date.now(),
      duration: event.duration_ms ?? (Date.now() - (execution.startTime || Date.now())),
    }));
  }, [updateExecution]);

  const handleX402Payment = useCallback((data: unknown) => {
    const event = data as X402PaymentEvent;
    setPayments((prev) => [...prev, event]);
  }, []);

  useEffect(() => {
    on('execution.started', handleExecutionStarted);
    on('execution.thinking', handleExecutionThinking);
    on('execution.task_started', handleTaskStarted);
    on('execution.task_updated', handleTaskUpdated);
    on('execution.task_completed', handleTaskCompleted);
    on('execution.completed', handleExecutionCompleted);
    on('x402.payment', handleX402Payment);

    return () => {
      off('execution.started', handleExecutionStarted);
      off('execution.thinking', handleExecutionThinking);
      off('execution.task_started', handleTaskStarted);
      off('execution.task_updated', handleTaskUpdated);
      off('execution.task_completed', handleTaskCompleted);
      off('execution.completed', handleExecutionCompleted);
      off('x402.payment', handleX402Payment);
    };
  }, [on, off, handleExecutionStarted, handleExecutionThinking, handleTaskStarted, handleTaskUpdated, handleTaskCompleted, handleExecutionCompleted, handleX402Payment]);

  const toggleCollapse = (taskId: string) => {
    setCollapsed((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(taskId)) {
        newSet.delete(taskId);
      } else {
        newSet.add(taskId);
      }
      return newSet;
    });
  };

  const formatDuration = (ms?: number): string => {
    if (!ms) return '';
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const formatTimestamp = (ts: string): string => {
    const date = new Date(ts);
    return date.toLocaleTimeString();
  };

  const totalPayments = payments.reduce((sum, p) => {
    const amount = parseFloat(p.amount_formatted || '0');
    return sum + amount;
  }, 0);

  const renderTask = (task: DebugTask, depth: number = 0): JSX.Element => {
    const hasChildren = task.children.length > 0;
    const isCollapsed = collapsed.has(task.id);

    const statusIcon = {
      pending: <span className="text-slate-500">○</span>,
      in_progress: <span className="text-cyan-400 animate-pulse">●</span>,
      completed: <span className="text-green-400">✓</span>,
      error: <span className="text-red-400">✗</span>,
    };

    const typeColors: Record<string, string> = {
      tool: 'text-purple-400',
      thinking: 'text-yellow-400',
      agent: 'text-blue-400',
      execution: 'text-cyan-400',
      plan: 'text-orange-400',
    };

    const taskText = task.status === 'in_progress' && task.activeForm
      ? task.activeForm
      : task.description || task.name;

    return (
      <div key={task.id} className="border-l border-slate-700 ml-2">
        <div
          className={clsx(
            'py-2 px-3 hover:bg-slate-800/50 text-sm',
            task.status === 'in_progress' && 'bg-slate-800/30'
          )}
        >
          {/* Header row with controls */}
          <div className="flex items-center gap-2 mb-1">
            {hasChildren && (
              <button
                onClick={() => toggleCollapse(task.id)}
                className="p-0.5 hover:bg-slate-700 rounded shrink-0"
              >
                {isCollapsed ? (
                  <ChevronRight className="w-3 h-3 text-slate-500" />
                ) : (
                  <ChevronDown className="w-3 h-3 text-slate-500" />
                )}
              </button>
            )}
            {!hasChildren && <div className="w-4 shrink-0" />}

            <div className="shrink-0">{statusIcon[task.status]}</div>

            {/* Task type badge */}
            {task.taskType && (
              <span className={clsx(
                'text-xs px-1.5 py-0.5 rounded shrink-0',
                typeColors[task.taskType] || 'text-slate-400',
                'bg-slate-800'
              )}>
                {task.taskType}
              </span>
            )}

            {/* Metrics inline */}
            <div className="flex items-center gap-2 text-xs text-slate-500 ml-auto shrink-0">
              {task.duration && (
                <span className="flex items-center gap-1">
                  <Clock className="w-3 h-3" />
                  {formatDuration(task.duration)}
                </span>
              )}
              {task.toolsCount !== undefined && task.toolsCount > 0 && (
                <span className="flex items-center gap-1">
                  <Cpu className="w-3 h-3" />
                  {task.toolsCount}
                </span>
              )}
              {task.tokensUsed !== undefined && task.tokensUsed > 0 && (
                <span>
                  {task.tokensUsed >= 1000
                    ? `${(task.tokensUsed / 1000).toFixed(1)}k`
                    : task.tokensUsed}
                </span>
              )}
            </div>
          </div>

          {/* Full task description - ALWAYS SHOW COMPLETE TEXT */}
          <div
            className={clsx(
              'ml-6 font-mono text-xs whitespace-pre-wrap break-all',
              task.status === 'completed' && 'text-slate-400',
              task.status === 'error' && 'text-red-400',
              task.status === 'in_progress' && 'text-cyan-300',
              task.status === 'pending' && 'text-slate-300'
            )}
            style={{ wordBreak: 'break-word', overflowWrap: 'anywhere' }}
          >
            {taskText}
          </div>
        </div>

        {hasChildren && !isCollapsed && (
          <div className="ml-4">
            {task.children.map((child) => renderTask(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className={clsx(
      'bg-slate-900 border border-slate-700 rounded-lg',
      className
    )}>
      {/* Tab headers */}
      <div className="flex border-b border-slate-700 sticky top-0 bg-slate-900 z-10">
        <button
          onClick={() => setActiveTab('tasks')}
          className={clsx(
            'flex-1 px-4 py-2 text-sm font-medium transition-colors',
            activeTab === 'tasks'
              ? 'bg-slate-800 text-white border-b-2 border-cyan-500'
              : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
          )}
        >
          <Cpu className="w-4 h-4 inline mr-2" />
          Subtasks ({executions.size})
        </button>
        <button
          onClick={() => setActiveTab('payments')}
          className={clsx(
            'flex-1 px-4 py-2 text-sm font-medium transition-colors',
            activeTab === 'payments'
              ? 'bg-slate-800 text-white border-b-2 border-green-500'
              : 'text-slate-400 hover:text-white hover:bg-slate-800/50'
          )}
        >
          <DollarSign className="w-4 h-4 inline mr-2" />
          x402 Payments ({payments.length})
        </button>
      </div>

      {/* Tab content - scrollable with more height */}
      <div className="max-h-[500px] overflow-y-auto overflow-x-hidden">
        {activeTab === 'tasks' && (
          <div className="p-2">
            {executions.size === 0 ? (
              <div className="text-center text-slate-500 py-8">
                No executions yet. Send a message to see subtasks.
              </div>
            ) : (
              Array.from(executions.values()).map((execution) => (
                <div key={execution.id} className="mb-4">
                  {renderTask(execution)}
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'payments' && (
          <div className="p-2">
            {/* Total summary */}
            {payments.length > 0 && (
              <div className="mb-4 p-3 bg-slate-800 rounded-lg">
                <div className="text-sm text-slate-400">Total Spent</div>
                <div className="text-2xl font-bold text-green-400">
                  ${totalPayments.toFixed(6)} USDC
                </div>
              </div>
            )}

            {payments.length === 0 ? (
              <div className="text-center text-slate-500 py-8">
                No x402 payments yet. Payments appear when using pay-per-use AI endpoints.
              </div>
            ) : (
              <div className="space-y-2">
                {payments.map((payment, idx) => (
                  <div
                    key={idx}
                    className="p-3 bg-slate-800 rounded-lg border border-slate-700"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-lg font-semibold text-green-400">
                        ${payment.amount_formatted} {payment.asset}
                      </span>
                      <span className="text-xs text-slate-500">
                        {formatTimestamp(payment.timestamp)}
                      </span>
                    </div>

                    {/* Full details - NO TRUNCATION - use monospace for addresses */}
                    <div className="text-xs text-slate-400 space-y-1 font-mono" style={{ wordBreak: 'break-word', overflowWrap: 'anywhere' }}>
                      <div>
                        <span className="text-slate-500 font-sans">To: </span>
                        <span className="whitespace-pre-wrap">{payment.pay_to}</span>
                      </div>
                      <div>
                        <span className="text-slate-500 font-sans">Raw amount: </span>
                        <span className="whitespace-pre-wrap">{payment.amount}</span>
                      </div>
                      {payment.resource && (
                        <div>
                          <span className="text-slate-500 font-sans">Resource: </span>
                          <span className="whitespace-pre-wrap">{payment.resource}</span>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
