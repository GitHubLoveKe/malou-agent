// 日志测试脚本
import { logger } from '@/utils/logger';

console.log('=== 日志功能测试 ===');

// 测试不同级别的日志
logger.info('这是信息日志', 'test');
logger.warn('这是警告日志', 'test');
logger.error('这是错误日志', 'test');
logger.debug('这是调试日志', 'test');

// 测试获取日志
const recentLogs = logger.getRecentLogs(10);
console.log('最近10条日志:', recentLogs);

// 测试按级别过滤
const errorLogs = logger.getLogsByLevel('error');
console.log('错误日志数量:', errorLogs.length);

console.log('=== 日志功能测试完成 ===');