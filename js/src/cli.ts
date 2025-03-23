#!/usr/bin/env node

import { obfuscateWasmFile, ObfuscationLevel } from './index';
import * as path from 'path';
import * as fs from 'fs';

/**
 * 解析命令行参数
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const options: {
    level: ObfuscationLevel;
    preserveOriginal: boolean;
    help: boolean;
    version: boolean;
    verbose: boolean;
    output?: string;
    input?: string;
  } = {
    level: ObfuscationLevel.Medium,
    preserveOriginal: false,
    help: false,
    version: false,
    verbose: false
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    switch (arg) {
      case '-h':
      case '--help':
        options.help = true;
        break;
      case '-v':
      case '--version':
        options.version = true;
        break;
      case '--verbose':
        options.verbose = true;
        break;
      case '-l':
      case '--level':
        if (i + 1 < args.length) {
          const levelArg = args[++i].toLowerCase();
          switch (levelArg) {
            case 'low':
              options.level = ObfuscationLevel.Low;
              break;
            case 'medium':
              options.level = ObfuscationLevel.Medium;
              break;
            case 'high':
              options.level = ObfuscationLevel.High;
              break;
            case '0':
              options.level = ObfuscationLevel.Low;
              break;
            case '1':
              options.level = ObfuscationLevel.Medium;
              break;
            case '2':
              options.level = ObfuscationLevel.High;
              break;
            default:
              console.error(`Invalid level: ${levelArg}`);
              process.exit(1);
          }
        }
        break;
      case '-p':
      case '--preserve':
        options.preserveOriginal = true;
        break;
      case '-o':
      case '--output':
        if (i + 1 < args.length) {
          options.output = args[++i];
        }
        break;
      default:
        if (!options.input && !arg.startsWith('-')) {
          options.input = arg;
        }
        break;
    }
  }

  return options;
}

/**
 * 显示帮助信息
 */
function showHelp() {
  console.log(`
RusWaCipher CLI - WebAssembly混淆工具

用法: ruswacipher [选项] <input.wasm>

选项:
  -h, --help         显示此帮助信息
  -v, --version      显示版本信息
  -l, --level        设置混淆级别 (low, medium, high) 或 (0, 1, 2) [默认: medium]
  -p, --preserve     保留原始文件
  -o, --output       指定输出文件路径
  --verbose          显示详细日志

示例:
  ruswacipher input.wasm                # 使用中等混淆级别
  ruswacipher -l high input.wasm        # 使用高级混淆
  ruswacipher -p -o output.wasm input.wasm  # 指定输出文件并保留原始文件
  `);
}

/**
 * 显示版本信息
 */
function showVersion() {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8')
  );
  console.log(`RusWaCipher v${packageJson.version}`);
}

/**
 * 主函数
 */
function main() {
  const options = parseArgs();

  if (options.help) {
    showHelp();
    return;
  }

  if (options.version) {
    showVersion();
    return;
  }

  if (!options.input) {
    console.error('Error: 未指定输入文件');
    showHelp();
    process.exit(1);
  }

  try {
    if (options.verbose) {
      console.log(`混淆文件: ${options.input}`);
      console.log(`混淆级别: ${options.level}`);
      console.log(`保留原始文件: ${options.preserveOriginal}`);
      if (options.output) {
        console.log(`输出文件: ${options.output}`);
      }
    }

    const outputPath = obfuscateWasmFile(options.input, {
      level: options.level,
      preserveOriginal: options.preserveOriginal,
      outputPath: options.output
    });

    if (options.verbose) {
      console.log(`混淆成功，输出文件: ${outputPath}`);
    } else {
      console.log(`${outputPath}`);
    }
  } catch (error) {
    console.error(`错误: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
  }
}

// 执行程序
main(); 