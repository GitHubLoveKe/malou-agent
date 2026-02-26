// 日志服务
import { ref } from 'vue';

interface LogEntry {
  id: string;
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  source: string;
}

export class Logger {
  private static instance: Logger;
  private logs: LogEntry[] = [];
  private maxLogs = 1000;
  
  private constructor() {}
  
  public static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }
  
  public info(message: string, source: string = 'system') {
    this.addLog('info', message, source);
  }
  
  public warn(message: string, source: string = 'system') {
    this.addLog('warn', message, source);
  }
  
  public error(message: string, source: string = 'system') {
    this.addLog('error', message, source);
  }
  
  public debug(message: string, source: string = 'system') {
    this.addLog('debug', message, source);
  }
  
  private addLog(level: LogEntry['level'], message: string, source: string) {
    const log: LogEntry = {
      id: Date.now().toString(),
      timestamp: new Date(),
      level,
      message,
      source
    };
    
    this.logs.push(log);
    
    // 限制日志数量
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }
    
    // 在控制台输出
    console[level](`[${source}] ${message}`);
  }
  
  public getLogs(): LogEntry[] {
    return [...this.logs];
  }
  
  public clearLogs() {
    this.logs = [];
  }
  
  public getLogsByLevel(level: LogEntry['level']): LogEntry[] {
    return this.logs.filter(log => log.level === level);
  }
  
  public getRecentLogs(count: number = 50): LogEntry[] {
    return this.logs.slice(-count);
  }
}

// 全局日志实例
export const logger = Logger.getInstance();

// Vue组合式函数
export function useLogger() {
  const logs = ref<LogEntry[]>([]);
  
  const updateLogs = () => {
    logs.value = logger.getRecentLogs(100);
  };
  
  return {
    logs,
    updateLogs,
    logger
  };
}