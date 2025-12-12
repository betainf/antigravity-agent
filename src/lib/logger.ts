// Console API 风格的前端日志系统
import {LoggingCommands} from "@/commands/LoggingCommands.ts";
import {FrontendLogEntry} from "@/commands/types/logging.types.ts";

export class Logger {
  private sessionId = Math.random().toString(36).substring(7);

  info(...args: any[]) {
    this.writeLog('info', ...args);
  }

  warn(...args: any[]) {
    this.writeLog('warn', ...args);
  }

  error(...args: any[]) {
    this.writeLog('error', ...args);
  }

  debug(...args: any[]) {
    this.writeLog('debug', ...args);
  }

  private writeLog(level: FrontendLogEntry["level"], ...args: any[]) {
    // 处理多参数，类似 console.log
    let message = '';
    let details: any;

    if (args.length === 0) return;

    if (args.length === 1) {
      // 单个参数
      if (typeof args[0] === 'string') {
        message = args[0];
      } else {
        message = '日志数据';
        details = args[0];
      }
    } else {
      // 多个参数，第一个作为消息，其余作为详情
      message = String(args[0]);
      if (args.length > 1) {
        details = args.slice(1);
        // 如果只有一个额外参数且是对象，直接使用
        if (details.length === 1 && typeof details[0] === 'object') {
          details = details[0];
        } else if (details.length === 1) {
          details = details[0];
        }
      }
    }

    const logEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      details: details ? JSON.stringify(details) : undefined,
      sessionId: this.sessionId
    };

    LoggingCommands.writeFrontendLog(logEntry);

    // 同时输出到浏览器控制台
    const consoleMethod = console[level as keyof Console] || console.log;
    // @ts-ignore
    consoleMethod(...args);
  }
}

export const logger = new Logger();
