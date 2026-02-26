import { invoke } from '@tauri-apps/api/core';
export * from './config-api';

// ============ 数据类型定义 ============

export interface Conversation {
  id: string;
  title: string;
  model_id: string | null;
  total_tokens: number;
  message_count: number;
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  model_id: string | null;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  created_at: string;
}

export interface SendMessageRequest {
  conversation_id: string;
  content: string;
}

export interface SendMessageResponse {
  id: string;
  content: string;
  role: string;
  conversation_id: string;
  timestamp: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface TokenSummary {
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_tokens: number;
  total_requests: number;
}

export interface DailyTokenUsage {
  date: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  request_count: number;
}

export interface OpenAIConfig {
  api_key: string;
  api_base: string;
  model: string;
  temperature: number;
}

// ============ 会话管理 API ============

/**
 * 创建新会话
 */
export async function createConversation(title: string): Promise<Conversation> {
  return await invoke('create_conversation', { title });
}

/**
 * 获取会话列表
 */
export async function listConversations(limit?: number, offset?: number): Promise<Conversation[]> {
  return await invoke('list_conversations', { limit, offset });
}

/**
 * 获取单个会话
 */
export async function getConversation(id: string): Promise<Conversation | null> {
  return await invoke('get_conversation', { id });
}

/**
 * 更新会话标题
 */
export async function updateConversationTitle(id: string, title: string): Promise<void> {
  return await invoke('update_conversation_title', { id, title });
}

/**
 * 删除会话
 */
export async function deleteConversation(id: string): Promise<boolean> {
  return await invoke('delete_conversation', { id });
}

// ============ 消息管理 API ============

/**
 * 发送消息
 */
export async function sendMessage(request: SendMessageRequest): Promise<SendMessageResponse> {
  return await invoke('send_message', { request });
}

/**
 * 获取会话消息
 */
export async function getConversationMessages(
  conversationId: string, 
  limit?: number, 
  offset?: number
): Promise<Message[]> {
  return await invoke('get_conversation_messages', { 
    conversationId, 
    limit, 
    offset 
  });
}

/**
 * 清空会话消息
 */
export async function clearConversationMessages(conversationId: string): Promise<number> {
  return await invoke('clear_conversation_messages', { conversationId });
}

// ============ Token 统计 API ============

/**
 * 获取 Token 使用汇总
 */
export async function getTokenUsageSummary(
  startDate?: string, 
  endDate?: string
): Promise<TokenSummary> {
  return await invoke('get_token_usage_summary', { startDate, endDate });
}

/**
 * 获取 Token 使用趋势
 */
export async function getTokenUsageTrend(days?: number): Promise<DailyTokenUsage[]> {
  return await invoke('get_token_usage_trend', { days });
}

// ============ OpenAI 配置 API ============

/**
 * 获取 OpenAI 配置
 */
export async function getOpenAIConfig(): Promise<OpenAIConfig> {
  return await invoke('get_openai_config');
}

/**
 * 保存 OpenAI 配置
 */
export async function saveOpenAIConfig(config: OpenAIConfig): Promise<void> {
  return await invoke('save_openai_config', { config });
}

// ============ 数据库测试 API ============

/**
 * 测试数据库连接
 */
export async function testDatabase(): Promise<string> {
  return await invoke('test_database');
}

// ============ 旧版 API（保持兼容） ============

export interface Document {
  id: string;
  title: string;
  content: string;
  embedding?: number[];
  metadata?: Record<string, any>;
  createdAt: string;
  updatedAt: string;
}

export interface DocumentCreate {
  title: string;
  content: string;
  embedding?: number[];
  metadata?: Record<string, any>;
}

export interface DocumentUpdate {
  title?: string;
  content?: string;
  embedding?: number[];
  metadata?: Record<string, any>;
}

export async function searchKnowledge(query: string): Promise<string[]> {
  return await invoke('search_knowledge', { query });
}

export async function runSkill(skillName: string, params: any): Promise<any> {
  return await invoke('run_skill', { skillName, params });
}

export async function embedText(text: string): Promise<number[]> {
  return await invoke('embed_text', { text });
}

// 文档管理 API（待实现）
export async function createDocument(_doc: DocumentCreate): Promise<Document> {
  throw new Error('Not implemented yet');
}

export async function getDocument(_id: string): Promise<Document | null> {
  throw new Error('Not implemented yet');
}

export async function updateDocument(_id: string, _doc: DocumentUpdate): Promise<Document | null> {
  throw new Error('Not implemented yet');
}

export async function deleteDocument(_id: string): Promise<boolean> {
  throw new Error('Not implemented yet');
}

export async function listDocuments(_limit: number = 10, _offset: number = 0): Promise<Document[]> {
  throw new Error('Not implemented yet');
}

// 清理聊天历史（旧版兼容）
export async function clearChatHistory(): Promise<void> {
  // 这个 API 已废弃，使用 clearConversationMessages 代替
  console.warn('clearChatHistory is deprecated, use clearConversationMessages instead');
}
