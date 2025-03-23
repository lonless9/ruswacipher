(module
  ;; 导出基本算术函数
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    (i32.add 
      (local.get $a) 
      (local.get $b)
    )
  )
  
  (func $subtract (export "subtract") (param $a i32) (param $b i32) (result i32)
    (i32.sub 
      (local.get $a) 
      (local.get $b)
    )
  )
  
  (func $multiply (export "multiply") (param $a i32) (param $b i32) (result i32)
    (i32.mul 
      (local.get $a) 
      (local.get $b)
    )
  )
  
  ;; 除法需要特别处理除以零的情况
  (func $divide (export "divide") (param $a i32) (param $b i32) (result f32)
    (if (result f32)
      (i32.eqz (local.get $b))
      (then
        ;; 除数为零，返回NaN
        (f32.const nan)
      )
      (else
        ;; 正常除法
        (f32.div 
          (f32.convert_i32_s (local.get $a))
          (f32.convert_i32_s (local.get $b))
        )
      )
    )
  )
  
  ;; 复杂数学函数 (从complex.wat中保留)
  (func $complex_math (export "complex_math") (param $a i32) (param $b i32) (param $c i32) (result i32)
    (local $temp1 i32)
    (local $temp2 i32)
    (local $temp3 i32)
    (local $result i32)
    
    ;; 复杂计算1: temp1 = (a * b) + (c * 2)
    (local.set $temp1
      (i32.add
        (i32.mul
          (local.get $a)
          (local.get $b)
        )
        (i32.mul
          (local.get $c)
          (i32.const 2)
        )
      )
    )
    
    ;; 复杂计算2: temp2 = (a + b) * (c - 3)
    (local.set $temp2
      (i32.mul
        (i32.add
          (local.get $a)
          (local.get $b)
        )
        (i32.sub
          (local.get $c)
          (i32.const 3)
        )
      )
    )
    
    ;; 复杂计算3: temp3 = (temp1 * temp2) / (a + 1)
    (local.set $temp3
      (i32.div_s
        (i32.mul
          (local.get $temp1)
          (local.get $temp2)
        )
        (i32.add
          (local.get $a)
          (i32.const 1)
        )
      )
    )
    
    ;; 结果计算: result = (temp3 & 0xFFFF) | ((temp1 + temp2) << 16)
    (local.set $result
      (i32.or
        (i32.and
          (local.get $temp3)
          (i32.const 0xFFFF)
        )
        (i32.shl
          (i32.add
            (local.get $temp1)
            (local.get $temp2)
          )
          (i32.const 16)
        )
      )
    )
    
    ;; 返回最终结果
    (local.get $result)
  )
  
  ;; 内存管理功能
  ;; 分配内存
  (func $allocate (export "allocate") (param $size i32) (result i32)
    ;; 内存分配 - 返回当前内存大小作为指针，然后增加内存
    (local $current i32)
    (local $pages_needed i32)
    (local $current_pages i32)
    
    ;; 获取当前内存大小（以字节为单位）
    (local.set $current_pages (memory.size))
    (local.set $current (i32.mul (local.get $current_pages) (i32.const 65536)))
    
    ;; 计算需要增加的页数 (1页 = 64KB)
    ;; (size + 65535) / 65536
    (local.set $pages_needed 
      (i32.div_u 
        (i32.add 
          (local.get $size)
          (i32.const 65535)
        )
        (i32.const 65536)
      )
    )
    
    ;; 增加内存
    (drop (memory.grow (local.get $pages_needed)))
    
    ;; 返回分配的指针
    (local.get $current)
  )
  
  ;; 释放内存（在WebAssembly中是空操作，但我们需要提供这个接口）
  (func $deallocate (export "deallocate") (param $ptr i32) (param $size i32)
    ;; 在这个简单实现中，我们不执行实际的内存释放
    ;; 在实际应用中，这里应该有内存管理逻辑
  )
  
  ;; 密码强度检查函数
  (func $check_password_strength (export "check_password_strength") (param $password_ptr i32) (result i32)
    (local $score i32)       ;; 密码强度分数
    (local $length i32)      ;; 密码长度
    (local $i i32)           ;; 循环计数器
    (local $char i32)        ;; 当前字符
    (local $has_lowercase i32)  ;; 是否有小写字母
    (local $has_uppercase i32)  ;; 是否有大写字母
    (local $has_digit i32)      ;; 是否有数字
    (local $has_special i32)    ;; 是否有特殊字符
    
    ;; 初始化
    (local.set $score (i32.const 0))
    (local.set $length (i32.const 0))
    (local.set $has_lowercase (i32.const 0))
    (local.set $has_uppercase (i32.const 0))
    (local.set $has_digit (i32.const 0))
    (local.set $has_special (i32.const 0))
    
    ;; 检查密码指针是否为空
    (if (i32.eqz (local.get $password_ptr))
      (then
        ;; 返回0分，表示非常弱
        (return (i32.const 0))
      )
    )
    
    ;; 计算密码长度并检查字符类型
    (local.set $i (i32.const 0))
    (block $done
      (loop $check_chars
        ;; 读取当前字符
        (local.set $char (i32.load8_u (i32.add (local.get $password_ptr) (local.get $i))))
        
        ;; 如果遇到null终止符，退出循环
        (br_if $done (i32.eqz (local.get $char)))
        
        ;; 增加长度计数
        (local.set $length (i32.add (local.get $length) (i32.const 1)))
        
        ;; 检查字符类型
        ;; 小写字母: a-z (97-122)
        (if (i32.and 
              (i32.ge_u (local.get $char) (i32.const 97))
              (i32.le_u (local.get $char) (i32.const 122))
            )
          (then
            (local.set $has_lowercase (i32.const 1))
          )
        )
        
        ;; 大写字母: A-Z (65-90)
        (if (i32.and 
              (i32.ge_u (local.get $char) (i32.const 65))
              (i32.le_u (local.get $char) (i32.const 90))
            )
          (then
            (local.set $has_uppercase (i32.const 1))
          )
        )
        
        ;; 数字: 0-9 (48-57)
        (if (i32.and 
              (i32.ge_u (local.get $char) (i32.const 48))
              (i32.le_u (local.get $char) (i32.const 57))
            )
          (then
            (local.set $has_digit (i32.const 1))
          )
        )
        
        ;; 特殊字符: 非字母数字
        (if (i32.and
              (i32.ne (local.get $has_lowercase) (i32.const 1))
              (i32.and
                (i32.ne (local.get $has_uppercase) (i32.const 1))
                (i32.ne (local.get $has_digit) (i32.const 1))
              )
            )
          (then
            (local.set $has_special (i32.const 1))
          )
        )
        
        ;; 增加循环计数器
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        
        ;; 继续循环
        (br $check_chars)
      )
    )
    
    ;; 密码长度检查
    (if (i32.lt_u (local.get $length) (i32.const 6))
      (then
        (return (i32.const 0)) ;; 非常弱
      )
    )
    
    ;; 根据长度评分
    (if (i32.ge_u (local.get $length) (i32.const 8))
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    (if (i32.ge_u (local.get $length) (i32.const 12))
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    
    ;; 根据字符类型评分
    (if (local.get $has_lowercase)
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    (if (local.get $has_uppercase)
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    (if (local.get $has_digit)
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    (if (local.get $has_special)
      (then
        (local.set $score (i32.add (local.get $score) (i32.const 1)))
      )
    )
    
    ;; 返回最终评分
    (local.get $score)
  )
  
  ;; JSON解析函数（简单实现）
  (func $parse_json (export "parse_json") (param $json_ptr i32) (result i32)
    (local $i i32)              ;; 循环计数器
    (local $char i32)           ;; 当前字符
    (local $brace_count i32)    ;; 花括号计数
    (local $result_ptr i32)     ;; 结果字符串指针
    (local $is_valid i32)       ;; JSON是否有效
    
    ;; 初始化
    (local.set $brace_count (i32.const 0))
    (local.set $is_valid (i32.const 1))  ;; 假设一开始是有效的
    
    ;; 检查JSON指针是否为空
    (if (i32.eqz (local.get $json_ptr))
      (then
        ;; 分配内存创建错误消息
        (local.set $result_ptr (call $allocate_string (i32.const 28)))  ;; "JSON string cannot be empty"长度
        
        ;; 组装错误消息
        (call $write_string_char (local.get $result_ptr) (i32.const 0) (i32.const 74))  ;; J
        (call $write_string_char (local.get $result_ptr) (i32.const 1) (i32.const 83))  ;; S
        (call $write_string_char (local.get $result_ptr) (i32.const 2) (i32.const 79))  ;; O
        (call $write_string_char (local.get $result_ptr) (i32.const 3) (i32.const 78))  ;; N
        (call $write_string_char (local.get $result_ptr) (i32.const 4) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 5) (i32.const 115))  ;; s
        (call $write_string_char (local.get $result_ptr) (i32.const 6) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 7) (i32.const 114))  ;; r
        (call $write_string_char (local.get $result_ptr) (i32.const 8) (i32.const 105))  ;; i
        (call $write_string_char (local.get $result_ptr) (i32.const 9) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 10) (i32.const 103))  ;; g
        (call $write_string_char (local.get $result_ptr) (i32.const 11) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 12) (i32.const 99))  ;; c
        (call $write_string_char (local.get $result_ptr) (i32.const 13) (i32.const 97))  ;; a
        (call $write_string_char (local.get $result_ptr) (i32.const 14) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 15) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 16) (i32.const 111))  ;; o
        (call $write_string_char (local.get $result_ptr) (i32.const 17) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 18) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 19) (i32.const 98))  ;; b
        (call $write_string_char (local.get $result_ptr) (i32.const 20) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 21) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 22) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 23) (i32.const 109))  ;; m
        (call $write_string_char (local.get $result_ptr) (i32.const 24) (i32.const 112))  ;; p
        (call $write_string_char (local.get $result_ptr) (i32.const 25) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 26) (i32.const 121))  ;; y
        (call $write_string_char (local.get $result_ptr) (i32.const 27) (i32.const 0))   ;; null终止符
        
        (return (local.get $result_ptr))
      )
    )
    
    ;; 检查JSON格式（只检查花括号匹配）
    (local.set $i (i32.const 0))
    (block $done
      (loop $check_json
        ;; 读取当前字符
        (local.set $char (i32.load8_u (i32.add (local.get $json_ptr) (local.get $i))))
        
        ;; 如果遇到null终止符，退出循环
        (br_if $done (i32.eqz (local.get $char)))
        
        ;; 检查花括号
        (if (i32.eq (local.get $char) (i32.const 123))  ;; '{'
          (then
            (local.set $brace_count (i32.add (local.get $brace_count) (i32.const 1)))
          )
        )
        (if (i32.eq (local.get $char) (i32.const 125))  ;; '}'
          (then
            (local.set $brace_count (i32.sub (local.get $brace_count) (i32.const 1)))
            
            ;; 如果brace_count小于0，则JSON无效
            (if (i32.lt_s (local.get $brace_count) (i32.const 0))
              (then
                (local.set $is_valid (i32.const 0))
                (br $done)
              )
            )
          )
        )
        
        ;; 增加循环计数器
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        
        ;; 继续循环
        (br $check_json)
      )
    )
    
    ;; 检查最终的括号平衡情况
    (if (i32.ne (local.get $brace_count) (i32.const 0))
      (then
        (local.set $is_valid (i32.const 0))
      )
    )
    
    ;; 根据验证结果返回不同的消息
    (if (result i32) (local.get $is_valid)
      (then
        ;; JSON有效
        (local.set $result_ptr (call $allocate_string (i32.const 30)))  ;; "Valid JSON with X object(s)"大约长度
        
        ;; 组装成功消息
        (call $write_string_char (local.get $result_ptr) (i32.const 0) (i32.const 86))   ;; V
        (call $write_string_char (local.get $result_ptr) (i32.const 1) (i32.const 97))   ;; a
        (call $write_string_char (local.get $result_ptr) (i32.const 2) (i32.const 108))  ;; l
        (call $write_string_char (local.get $result_ptr) (i32.const 3) (i32.const 105))  ;; i
        (call $write_string_char (local.get $result_ptr) (i32.const 4) (i32.const 100))  ;; d
        (call $write_string_char (local.get $result_ptr) (i32.const 5) (i32.const 32))   ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 6) (i32.const 74))   ;; J
        (call $write_string_char (local.get $result_ptr) (i32.const 7) (i32.const 83))   ;; S
        (call $write_string_char (local.get $result_ptr) (i32.const 8) (i32.const 79))   ;; O
        (call $write_string_char (local.get $result_ptr) (i32.const 9) (i32.const 78))   ;; N
        
        ;; null终止符
        (call $write_string_char (local.get $result_ptr) (i32.const 10) (i32.const 0))
        
        (local.get $result_ptr)
      )
      (else
        ;; JSON无效
        (local.set $result_ptr (call $allocate_string (i32.const 40)))  ;; 错误消息大约长度
        
        ;; 组装错误消息
        (call $write_string_char (local.get $result_ptr) (i32.const 0) (i32.const 73))   ;; I
        (call $write_string_char (local.get $result_ptr) (i32.const 1) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 2) (i32.const 118))  ;; v
        (call $write_string_char (local.get $result_ptr) (i32.const 3) (i32.const 97))   ;; a
        (call $write_string_char (local.get $result_ptr) (i32.const 4) (i32.const 108))  ;; l
        (call $write_string_char (local.get $result_ptr) (i32.const 5) (i32.const 105))  ;; i
        (call $write_string_char (local.get $result_ptr) (i32.const 6) (i32.const 100))  ;; d
        (call $write_string_char (local.get $result_ptr) (i32.const 7) (i32.const 32))   ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 8) (i32.const 74))   ;; J
        (call $write_string_char (local.get $result_ptr) (i32.const 9) (i32.const 83))   ;; S
        (call $write_string_char (local.get $result_ptr) (i32.const 10) (i32.const 79))  ;; O
        (call $write_string_char (local.get $result_ptr) (i32.const 11) (i32.const 78))  ;; N
        
        ;; null终止符
        (call $write_string_char (local.get $result_ptr) (i32.const 12) (i32.const 0))
        
        (local.get $result_ptr)
      )
    )
  )
  
  ;; 辅助函数：分配字符串内存
  (func $allocate_string (param $length i32) (result i32)
    ;; 分配内存，加1是为了存储null终止符
    (call $allocate (i32.add (local.get $length) (i32.const 1)))
  )
  
  ;; 辅助函数：写入字符到字符串
  (func $write_string_char (param $str_ptr i32) (param $index i32) (param $char i32)
    (i32.store8 
      (i32.add (local.get $str_ptr) (local.get $index))
      (local.get $char)
    )
  )
  
  ;; 字符串加密函数（使用改进的XOR加密）
  (func $encrypt_string (export "encrypt_string") (param $text_ptr i32) (param $key_ptr i32) (result i32)
    (local $i i32)              ;; 文本循环计数器
    (local $j i32)              ;; 密钥循环计数器
    (local $text_char i32)      ;; 当前文本字符
    (local $key_char i32)       ;; 当前密钥字符
    (local $encrypted_char i32) ;; 加密后的字符
    (local $text_len i32)       ;; 文本长度
    (local $key_len i32)        ;; 密钥长度
    (local $result_ptr i32)     ;; 结果字符串指针
    (local $encoded_len i32)    ;; 编码后字符串长度
    
    ;; 检查文本指针是否为空
    (if (i32.eqz (local.get $text_ptr))
      (then
        ;; 返回错误消息
        (local.set $result_ptr (call $allocate_string (i32.const 22)))  ;; "Text cannot be empty"
        (call $write_error_string (local.get $result_ptr) (i32.const 84) (i32.const 101) (i32.const 120) (i32.const 116))  ;; "Text"
        (return (local.get $result_ptr))
      )
    )
    
    ;; 检查密钥指针是否为空
    (if (i32.eqz (local.get $key_ptr))
      (then
        ;; 返回错误消息
        (local.set $result_ptr (call $allocate_string (i32.const 22)))  ;; "Key cannot be empty"
        (call $write_error_string (local.get $result_ptr) (i32.const 75) (i32.const 101) (i32.const 121) (i32.const 32))  ;; "Key "
        (return (local.get $result_ptr))
      )
    )
    
    ;; 检查密钥长度是否为0
    (local.set $key_len (call $string_length (local.get $key_ptr)))
    (if (i32.eqz (local.get $key_len))
      (then
        ;; 返回错误消息
        (local.set $result_ptr (call $allocate_string (i32.const 25)))  ;; "Key must not be empty"
        
        ;; 组装错误消息
        (call $write_string_char (local.get $result_ptr) (i32.const 0) (i32.const 75))  ;; K
        (call $write_string_char (local.get $result_ptr) (i32.const 1) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 2) (i32.const 121))  ;; y
        (call $write_string_char (local.get $result_ptr) (i32.const 3) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 4) (i32.const 109))  ;; m
        (call $write_string_char (local.get $result_ptr) (i32.const 5) (i32.const 117))  ;; u
        (call $write_string_char (local.get $result_ptr) (i32.const 6) (i32.const 115))  ;; s
        (call $write_string_char (local.get $result_ptr) (i32.const 7) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 8) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 9) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 10) (i32.const 111))  ;; o
        (call $write_string_char (local.get $result_ptr) (i32.const 11) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 12) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 13) (i32.const 98))  ;; b
        (call $write_string_char (local.get $result_ptr) (i32.const 14) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 15) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 16) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 17) (i32.const 109))  ;; m
        (call $write_string_char (local.get $result_ptr) (i32.const 18) (i32.const 112))  ;; p
        (call $write_string_char (local.get $result_ptr) (i32.const 19) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 20) (i32.const 121))  ;; y
        (call $write_string_char (local.get $result_ptr) (i32.const 21) (i32.const 0))  ;; null终止符
        
        (return (local.get $result_ptr))
      )
    )
    
    ;; 获取文本长度
    (local.set $text_len (call $string_length (local.get $text_ptr)))
    
    ;; 检查文本长度是否为0
    (if (i32.eqz (local.get $text_len))
      (then
        ;; 返回错误消息
        (local.set $result_ptr (call $allocate_string (i32.const 25)))  ;; "Text must not be empty"
        
        ;; 组装错误消息
        (call $write_string_char (local.get $result_ptr) (i32.const 0) (i32.const 84))  ;; T
        (call $write_string_char (local.get $result_ptr) (i32.const 1) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 2) (i32.const 120))  ;; x
        (call $write_string_char (local.get $result_ptr) (i32.const 3) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 4) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 5) (i32.const 109))  ;; m
        (call $write_string_char (local.get $result_ptr) (i32.const 6) (i32.const 117))  ;; u
        (call $write_string_char (local.get $result_ptr) (i32.const 7) (i32.const 115))  ;; s
        (call $write_string_char (local.get $result_ptr) (i32.const 8) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 9) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 10) (i32.const 110))  ;; n
        (call $write_string_char (local.get $result_ptr) (i32.const 11) (i32.const 111))  ;; o
        (call $write_string_char (local.get $result_ptr) (i32.const 12) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 13) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 14) (i32.const 98))  ;; b
        (call $write_string_char (local.get $result_ptr) (i32.const 15) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 16) (i32.const 32))  ;; 空格
        (call $write_string_char (local.get $result_ptr) (i32.const 17) (i32.const 101))  ;; e
        (call $write_string_char (local.get $result_ptr) (i32.const 18) (i32.const 109))  ;; m
        (call $write_string_char (local.get $result_ptr) (i32.const 19) (i32.const 112))  ;; p
        (call $write_string_char (local.get $result_ptr) (i32.const 20) (i32.const 116))  ;; t
        (call $write_string_char (local.get $result_ptr) (i32.const 21) (i32.const 121))  ;; y
        (call $write_string_char (local.get $result_ptr) (i32.const 22) (i32.const 0))  ;; null终止符
        
        (return (local.get $result_ptr))
      )
    )
    
    ;; 为了安全起见，为结果分配更多空间（原文本长度的2倍）以处理可能的边界情况
    (local.set $encoded_len (i32.mul (local.get $text_len) (i32.const 2)))
    (local.set $result_ptr (call $allocate_string (local.get $encoded_len)))
    
    ;; 进行XOR加密
    (local.set $i (i32.const 0))
    (local.set $j (i32.const 0))
    
    (block $encrypt_done
      (loop $encrypt_loop
        ;; 如果文本已处理完，退出循环
        (br_if $encrypt_done (i32.ge_u (local.get $i) (local.get $text_len)))
        
        ;; 获取当前字符
        (local.set $text_char (i32.load8_u (i32.add (local.get $text_ptr) (local.get $i))))
        (local.set $key_char (i32.load8_u (i32.add (local.get $key_ptr) (local.get $j))))
        
        ;; 使用XOR操作加密字符
        (local.set $encrypted_char (i32.xor (local.get $text_char) (local.get $key_char)))
        
        ;; 确保结果是可打印ASCII字符 (32-126) 或常用控制字符 (9-13)
        ;; 如果不是，则将其转换为可打印范围内的字符
        (if (i32.or
              (i32.and
                (i32.ge_u (local.get $encrypted_char) (i32.const 32))
                (i32.le_u (local.get $encrypted_char) (i32.const 126))
              )
              (i32.and
                (i32.ge_u (local.get $encrypted_char) (i32.const 9))
                (i32.le_u (local.get $encrypted_char) (i32.const 13))
              )
            )
          (then
            ;; 字符已经是可打印字符，保持不变
          )
          (else
            ;; 将字符转换为可打印范围：(char % 95) + 32
            ;; 这会将字符映射到ASCII可打印范围 (32-126)
            (local.set $encrypted_char 
              (i32.add
                (i32.rem_u 
                  (local.get $encrypted_char)
                  (i32.const 95)
                )
                (i32.const 32)
              )
            )
          )
        )
        
        ;; 写入加密字符
        (call $write_string_char (local.get $result_ptr) (local.get $i) (local.get $encrypted_char))
        
        ;; 递增文本计数器
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        
        ;; 递增密钥计数器，并在需要时循环使用密钥
        (local.set $j (i32.add (local.get $j) (i32.const 1)))
        (if (i32.ge_u (local.get $j) (local.get $key_len))
          (then
            (local.set $j (i32.const 0))
          )
        )
        
        ;; 继续循环
        (br $encrypt_loop)
      )
    )
    
    ;; 添加null终止符
    (call $write_string_char (local.get $result_ptr) (local.get $text_len) (i32.const 0))
    
    ;; 返回加密结果
    (local.get $result_ptr)
  )
  
  ;; 字符串解密函数（与加密功能相同，因为XOR加密和解密是相同的操作）
  (func $decrypt_string (export "decrypt_string") (param $encrypted_ptr i32) (param $key_ptr i32) (result i32)
    ;; 直接复用加密函数，因为XOR加密和解密是相同的操作
    (call $encrypt_string (local.get $encrypted_ptr) (local.get $key_ptr))
  )
  
  ;; 辅助函数：获取字符串长度
  (func $string_length (param $str_ptr i32) (result i32)
    (local $length i32)
    (local $char i32)
    
    (local.set $length (i32.const 0))
    
    (block $done
      (loop $count
        ;; 读取当前字符
        (local.set $char (i32.load8_u (i32.add (local.get $str_ptr) (local.get $length))))
        
        ;; 如果是null终止符，退出循环
        (br_if $done (i32.eqz (local.get $char)))
        
        ;; 增加长度计数
        (local.set $length (i32.add (local.get $length) (i32.const 1)))
        
        ;; 继续循环
        (br $count)
      )
    )
    
    (local.get $length)
  )
  
  ;; 辅助函数：写入错误消息前缀
  (func $write_error_string (param $str_ptr i32) (param $c1 i32) (param $c2 i32) (param $c3 i32) (param $c4 i32)
    ;; 写入前4个字符（通常是错误类型，如"Text"或"Key"）
    (call $write_string_char (local.get $str_ptr) (i32.const 0) (local.get $c1))
    (call $write_string_char (local.get $str_ptr) (i32.const 1) (local.get $c2))
    (call $write_string_char (local.get $str_ptr) (i32.const 2) (local.get $c3))
    (call $write_string_char (local.get $str_ptr) (i32.const 3) (local.get $c4))
    
    ;; 写入" cannot be empty"
    (call $write_string_char (local.get $str_ptr) (i32.const 4) (i32.const 32))  ;; 空格
    (call $write_string_char (local.get $str_ptr) (i32.const 5) (i32.const 99))  ;; c
    (call $write_string_char (local.get $str_ptr) (i32.const 6) (i32.const 97))  ;; a
    (call $write_string_char (local.get $str_ptr) (i32.const 7) (i32.const 110))  ;; n
    (call $write_string_char (local.get $str_ptr) (i32.const 8) (i32.const 110))  ;; n
    (call $write_string_char (local.get $str_ptr) (i32.const 9) (i32.const 111))  ;; o
    (call $write_string_char (local.get $str_ptr) (i32.const 10) (i32.const 116))  ;; t
    (call $write_string_char (local.get $str_ptr) (i32.const 11) (i32.const 32))  ;; 空格
    (call $write_string_char (local.get $str_ptr) (i32.const 12) (i32.const 98))  ;; b
    (call $write_string_char (local.get $str_ptr) (i32.const 13) (i32.const 101))  ;; e
    (call $write_string_char (local.get $str_ptr) (i32.const 14) (i32.const 32))  ;; 空格
    (call $write_string_char (local.get $str_ptr) (i32.const 15) (i32.const 101))  ;; e
    (call $write_string_char (local.get $str_ptr) (i32.const 16) (i32.const 109))  ;; m
    (call $write_string_char (local.get $str_ptr) (i32.const 17) (i32.const 112))  ;; p
    (call $write_string_char (local.get $str_ptr) (i32.const 18) (i32.const 116))  ;; t
    (call $write_string_char (local.get $str_ptr) (i32.const 19) (i32.const 121))  ;; y
    (call $write_string_char (local.get $str_ptr) (i32.const 20) (i32.const 0))  ;; null终止符
  )
  
  ;; 内存导出
  (memory (export "memory") 1)
) 