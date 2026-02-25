import { invoke } from '@tauri-apps/api/core';

export interface MessageRequest {
  content: string;
  conversationId?: string;
}

export interface MessageResponse {
  content: string;
  conversationId: string;
  timestamp: string;
}

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

// 消息相关API
export async function sendMessage(request: MessageRequest): Promise<MessageResponse> {
  return await invoke('send_message', { request });
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

// 数据库相关API
export async function testDatabase(): Promise<string> {
  return await invoke('test_database');
}

// 文档管理API
export async function createDocument(doc: DocumentCreate): Promise<Document> {
  // 这里需要在后端添加对应的Tauri命令
  throw new Error('Not implemented yet');
}

export async function getDocument(id: string): Promise<Document | null> {
  throw new Error('Not implemented yet');
}

export async function updateDocument(id: string, doc: DocumentUpdate): Promise<Document | null> {
  throw new Error('Not implemented yet');
}

export async function deleteDocument(id: string): Promise<boolean> {
  throw new Error('Not implemented yet');
}

export async function listDocuments(limit: number = 10, offset: number = 0): Promise<Document[]> {
  throw new Error('Not implemented yet');
}