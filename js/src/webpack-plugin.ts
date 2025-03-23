import { Compiler } from 'webpack';
import { ObfuscationLevel, obfuscateWasmBuffer } from './index';
import * as path from 'path';

/**
 * RusWaCipher Webpack Plugin Options
 */
export interface RuswacipherPluginOptions {
  /**
   * Obfuscation level
   * @default ObfuscationLevel.Medium
   */
  level?: ObfuscationLevel | 'low' | 'medium' | 'high';
  
  /**
   * Match files to be obfuscated
   * @default /\.wasm$/
   */
  include?: RegExp | RegExp[] | string | string[];
  
  /**
   * Exclude files from obfuscation
   */
  exclude?: RegExp | RegExp[] | string | string[];
  
  /**
   * Whether to disable the plugin
   * @default false
   */
  disabled?: boolean;
  
  /**
   * Enable verbose logging
   * @default false
   */
  verbose?: boolean;
}

/**
 * RusWaCipher Webpack Plugin
 * Used to obfuscate WebAssembly files during the build process
 */
export class RuswacipherPlugin {
  private options: Required<RuswacipherPluginOptions>;
  
  constructor(options: RuswacipherPluginOptions = {}) {
    this.options = {
      level: ObfuscationLevel.Medium,
      include: /\.wasm$/,
      exclude: [],
      disabled: false,
      verbose: false,
      ...options
    };
    
    // Convert string level to enum
    if (typeof this.options.level === 'string') {
      switch (this.options.level.toLowerCase()) {
        case 'low':
          this.options.level = ObfuscationLevel.Low;
          break;
        case 'medium':
          this.options.level = ObfuscationLevel.Medium;
          break;
        case 'high':
          this.options.level = ObfuscationLevel.High;
          break;
        default:
          throw new Error(`Invalid obfuscation level: ${this.options.level}`);
      }
    }
  }
  
  /**
   * Check if file should be obfuscated
   */
  private shouldObfuscate(filePath: string): boolean {
    const { include, exclude } = this.options;
    
    // File path should match include
    const matchesInclude = (pattern: RegExp | string): boolean => {
      if (pattern instanceof RegExp) {
        return pattern.test(filePath);
      }
      return filePath.includes(pattern);
    };
    
    const isIncluded = Array.isArray(include)
      ? include.some(matchesInclude)
      : matchesInclude(include);
    
    if (!isIncluded) {
      return false;
    }
    
    // File path should not match exclude
    if (exclude) {
      const matchesExclude = (pattern: RegExp | string): boolean => {
        if (pattern instanceof RegExp) {
          return pattern.test(filePath);
        }
        return filePath.includes(pattern);
      };
      
      const isExcluded = Array.isArray(exclude)
        ? exclude.some(matchesExclude)
        : matchesExclude(exclude);
      
      if (isExcluded) {
        return false;
      }
    }
    
    return true;
  }
  
  /**
   * Log information
   */
  private log(message: string): void {
    if (this.options.verbose) {
      console.log(`[RusWaCipher] ${message}`);
    }
  }
  
  /**
   * Apply plugin to webpack
   */
  apply(compiler: Compiler): void {
    if (this.options.disabled) {
      this.log('Plugin is disabled, skipping.');
      return;
    }
    
    const pluginName = 'RuswacipherPlugin';
    
    // Use Webpack Plugin API
    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      // Webpack 5 version hooks
      if (compilation.hooks.processAssets) {
        const { Compilation } = compiler.webpack;
        
        compilation.hooks.processAssets.tap(
          {
            name: pluginName,
            stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE
          },
          (assets) => {
            this.log(`Processing ${Object.keys(assets).length} assets...`);
            
            for (const [fileName, asset] of Object.entries(assets)) {
              if (this.shouldObfuscate(fileName)) {
                this.log(`Obfuscating WASM file: ${fileName}`);
                
                // Get asset content
                const content = asset.source();
                const buffer = content instanceof Buffer
                  ? content
                  : Buffer.from(typeof content === 'string' ? content : content.toString());
                
                try {
                  // Obfuscate WebAssembly
                  const obfuscatedBuffer = obfuscateWasmBuffer(buffer, this.options.level);
                  
                  // Replace asset
                  compilation.updateAsset(
                    fileName,
                    new compiler.webpack.sources.RawSource(obfuscatedBuffer)
                  );
                  
                  this.log(`Successfully obfuscated: ${fileName} (Level: ${this.options.level})`);
                } catch (error) {
                  const errorMsg = error instanceof Error ? error.message : String(error);
                  compilation.warnings.push(
                    new Error(`[RusWaCipher] Failed to obfuscate ${fileName}: ${errorMsg}`)
                  );
                }
              }
            }
          }
        );
      } else {
        // Support for Webpack 4 and earlier versions
        compilation.hooks.additionalAssets.tap(pluginName, () => {
          this.log(`Processing assets (Webpack 4)...`);
          
          for (const [fileName, asset] of Object.entries(compilation.assets)) {
            if (this.shouldObfuscate(fileName)) {
              this.log(`Obfuscating WASM file: ${fileName}`);
              
              try {
                // Get asset content
                const content = asset.source();
                const buffer = content instanceof Buffer
                  ? content
                  : Buffer.from(typeof content === 'string' ? content : content.toString());
                
                // Obfuscate WebAssembly
                const obfuscatedBuffer = obfuscateWasmBuffer(buffer, this.options.level);
                
                // Replace asset
                compilation.assets[fileName] = {
                  source: () => obfuscatedBuffer,
                  size: () => obfuscatedBuffer.length
                };
                
                this.log(`Successfully obfuscated: ${fileName} (Level: ${this.options.level})`);
              } catch (error) {
                const errorMsg = error instanceof Error ? error.message : String(error);
                compilation.warnings.push(
                  new Error(`[RusWaCipher] Failed to obfuscate ${fileName}: ${errorMsg}`)
                );
              }
            }
          }
        });
      }
    });
  }
}

export default RuswacipherPlugin; 