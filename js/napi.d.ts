/**
 * RusWaCipher 原生模块类型声明
 */

/**
 * WebAssembly混淆级别
 */
export enum ObfuscationLevel {
  /**
   * 低级别混淆
   */
  Low = 0,
  /**
   * 中级别混淆
   */
  Medium = 1,
  /**
   * 高级别混淆
   */
  High = 2
}

/**
 * 加密算法类型
 */
export enum EncryptionAlgorithm {
  /**
   * AES-GCM算法
   */
  AesGcm = 0,
  /**
   * ChaCha20-Poly1305算法
   */
  ChaCha20Poly1305 = 1,
  /**
   * 支持所有算法
   */
  All = 2
}

/**
 * 混淆WebAssembly文件
 * 
 * @param inputPath 输入文件路径
 * @param outputPath 输出文件路径
 * @param level 混淆级别
 * @param algorithm 加密算法（可选）
 * @returns 操作是否成功
 */
export function obfuscateWasm(
  inputPath: string,
  outputPath: string,
  level: ObfuscationLevel,
  algorithm?: EncryptionAlgorithm
): boolean;

/**
 * 仅混淆WebAssembly文件（不加密）
 * 
 * @param inputPath 输入文件路径
 * @param outputPath 输出文件路径
 * @param level 混淆级别
 * @returns 操作是否成功
 */
export function obfuscateWasmOnly(
  inputPath: string,
  outputPath: string,
  level: ObfuscationLevel
): boolean;

/**
 * 获取RusWaCipher库版本
 * 
 * @returns 版本号
 */
export function getVersion(): string; 