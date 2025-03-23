import { obfuscateWasm as nativeObfuscateWasm, ObfuscationLevel, EncryptionAlgorithm } from '../napi';
import * as fs from 'fs';
import * as path from 'path';

/**
 * WebAssembly Obfuscation Level
 */
export enum WasmObfuscationLevel {
  /**
   * Low level obfuscation - Only applies basic variable renaming
   */
  Low = 0,
  
  /**
   * Medium level obfuscation - Variable renaming + Dead code insertion + Control flow obfuscation
   */
  Medium = 1,
  
  /**
   * High level obfuscation - Applies all protection techniques, including function splitting and virtualization
   */
  High = 2
}

/**
 * Options for WebAssembly file obfuscation
 */
export interface ObfuscateWasmOptions {
  /**
   * Obfuscation level
   * @default WasmObfuscationLevel.Medium
   */
  level?: WasmObfuscationLevel;
  
  /**
   * Encryption algorithm
   * @default EncryptionAlgorithm.AesGcm
   */
  algorithm?: EncryptionAlgorithm;
  
  /**
   * Whether to preserve the original file after processing
   * @default false
   */
  preserveOriginal?: boolean;
  
  /**
   * Custom output file path
   * If not provided, defaults to adding .obfuscated to the original filename
   */
  outputPath?: string;
}

/**
 * Obfuscate a WebAssembly file
 * 
 * @param inputPath Input WebAssembly file path
 * @param options Obfuscation options
 * @returns Path to the obfuscated file
 */
export function obfuscateWasmFile(
  inputPath: string,
  options: ObfuscateWasmOptions = {}
): string {
  const {
    level = WasmObfuscationLevel.Medium,
    algorithm = EncryptionAlgorithm.AesGcm,
    preserveOriginal = false,
    outputPath
  } = options;
  
  // Verify input file exists
  if (!fs.existsSync(inputPath)) {
    throw new Error(`Input file not found: ${inputPath}`);
  }
  
  // Determine output file path
  const resolvedOutputPath = outputPath || `${inputPath}.obfuscated`;
  
  // Call Native method for obfuscation, passing encryption algorithm
  nativeObfuscateWasm(inputPath, resolvedOutputPath, Number(level), algorithm);
  
  // Delete original file if not preserving
  if (!preserveOriginal && inputPath !== resolvedOutputPath) {
    fs.unlinkSync(inputPath);
  }
  
  return resolvedOutputPath;
}

/**
 * Obfuscate WebAssembly binary data
 * 
 * @param wasmBuffer Input WebAssembly binary data
 * @param level Obfuscation level
 * @param algorithm Encryption algorithm
 * @returns Obfuscated WebAssembly binary data
 */
export function obfuscateWasmBuffer(
  wasmBuffer: Buffer,
  level: WasmObfuscationLevel = WasmObfuscationLevel.Medium,
  algorithm: EncryptionAlgorithm = EncryptionAlgorithm.AesGcm
): Buffer {
  // Create temporary file
  const tempInputPath = path.join(
    fs.mkdtempSync('ruswacipher-'),
    'input.wasm'
  );
  const tempOutputPath = `${tempInputPath}.obfuscated`;
  
  try {
    // Write to temporary input file
    fs.writeFileSync(tempInputPath, wasmBuffer);
    
    // Call obfuscation, passing encryption algorithm
    nativeObfuscateWasm(tempInputPath, tempOutputPath, Number(level), algorithm);
    
    // Read output
    return fs.readFileSync(tempOutputPath);
  } finally {
    // Clean up temporary files
    try {
      if (fs.existsSync(tempInputPath)) fs.unlinkSync(tempInputPath);
      if (fs.existsSync(tempOutputPath)) fs.unlinkSync(tempOutputPath);
      
      // Try to delete temporary directory
      const tempDir = path.dirname(tempInputPath);
      fs.rmdirSync(tempDir);
    } catch (err) {
      // Ignore cleanup errors
    }
  }
}

export {
  WasmObfuscationLevel as ObfuscationLevel,
  obfuscateWasmFile as obfuscateWasm
};

// Re-export other functions from Native module
export { ObfuscationLevel as NativeObfuscationLevel, EncryptionAlgorithm } from '../napi'; 