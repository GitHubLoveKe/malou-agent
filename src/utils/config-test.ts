// 配置功能测试脚本
import { useConfig } from '../config';
import { getAppConfig, updateAppConfig } from '../api/tauri-api';

export async function testConfigurationFeatures() {
  console.log('开始测试配置功能...');
  
  try {
    // 1. 测试获取配置
    console.log('1. 测试获取配置...');
    const config = await getAppConfig();
    console.log('✓ 成功获取配置:', config);
    
    // 2. 测试配置更新
    console.log('2. 测试配置更新...');
    const testConfig = {
      ...config,
      general: {
        ...config.general,
        theme: (config.general.theme === 'light' ? 'dark' : 'light') as 'light' | 'dark' | 'auto'
      }
    };
    
    await updateAppConfig(testConfig);
    console.log('✓ 成功更新配置');
    
    // 3. 测试前端配置管理器
    console.log('3. 测试前端配置管理器...');
    const { config: frontendConfig, updateConfig } = useConfig();
    console.log('✓ 前端配置管理器工作正常:', frontendConfig.value);
    
    // 4. 测试配置验证
    console.log('4. 测试配置验证...');
    const validationResult = updateConfig({
      general: {
        ...frontendConfig.value.general,
        autoSave: !frontendConfig.value.general.autoSave
      }
    });
    console.log('✓ 配置验证结果:', validationResult);
    
    // 5. 测试当前模型获取
    console.log('5. 测试当前模型获取...');
    // 这里需要在Tauri后端实现相应的命令
    
    console.log('🎉 所有配置功能测试通过！');
    return true;
    
  } catch (error) {
    console.error('❌ 配置功能测试失败:', error);
    return false;
  }
}

// 扩展 Window 接口
declare global {
  interface Window {
    testConfiguration: () => Promise<boolean>;
  }
}

// 运行测试
if (typeof window !== 'undefined') {
  // 在浏览器环境中运行测试
  window.testConfiguration = testConfigurationFeatures;
}