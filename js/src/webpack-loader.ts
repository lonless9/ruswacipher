import { obfuscateWasmBuffer, ObfuscationLevel } from './index';
import { getOptions } from 'loader-utils';
import { validate } from 'schema-utils';
import * as webpack from 'webpack';

const schema = {
  type: 'object',
  properties: {
    level: {
      enum: ['low', 'medium', 'high', 0, 1, 2]
    },
    allowOriginalLoader: {
      type: 'boolean'
    }
  },
  additionalProperties: false
};

/**
 * RusWaCipher Webpack Loader
 * 用于在打包过程中混淆WebAssembly文件
 */
export default function loader(
  this: webpack.LoaderContext<any>,
  source: Buffer | string
): Buffer | string {
  // 获取loader选项
  const options = getOptions(this) || {};
  
  // 验证选项
  validate(schema, options, {
    name: 'RusWaCipher Loader',
    baseDataPath: 'options'
  });
  
  // 检查source类型
  if (typeof source === 'string') {
    // 原始Webpack wasm-loader将二进制数据作为Buffer传递
    // 如果我们得到字符串，这可能意味着已经被其他loader处理过
    if (options.allowOriginalLoader) {
      return source;
    }
    throw new Error(
      'RusWaCipher loader received string input. It should be used before any loader that converts binary to string.'
    );
  }
  
  // 设置混淆级别
  let level = ObfuscationLevel.Medium;
  
  if (options.level !== undefined) {
    if (typeof options.level === 'string') {
      switch (options.level.toLowerCase()) {
        case 'low':
          level = ObfuscationLevel.Low;
          break;
        case 'medium':
          level = ObfuscationLevel.Medium;
          break;
        case 'high':
          level = ObfuscationLevel.High;
          break;
        default:
          throw new Error(`Invalid obfuscation level: ${options.level}`);
      }
    } else {
      level = options.level;
    }
  }
  
  // 记录混淆信息
  if (this.emitWarning) {
    this.emitWarning(
      new Error(`RusWaCipher: Obfuscating ${this.resourcePath} at level ${level}`)
    );
  }
  
  // 进行混淆
  const obfuscatedBuffer = obfuscateWasmBuffer(source as Buffer, level);
  
  // 返回混淆后的WebAssembly二进制数据
  return obfuscatedBuffer;
} 