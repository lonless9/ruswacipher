(module
  ;; export function "add" - simple function
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    (i32.add 
      (local.get $a) 
      (local.get $b)
    )
  )
  
  ;; export function "complex_math" - large complex function
  (func $complex_math (export "complex_math") (param $a i32) (param $b i32) (param $c i32) (result i32)
    (local $temp1 i32)
    (local $temp2 i32)
    (local $temp3 i32)
    (local $result i32)
    
    ;; complex calculation 1: temp1 = (a * b) + (c * 2)
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
    
    ;; complex calculation 2: temp2 = (a + b) * (c - 3)
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
    
    ;; complex calculation 3: temp3 = (temp1 * temp2) / (a + 1)
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
    
    ;; result calculation: result = (temp3 & 0xFFFF) | ((temp1 + temp2) << 16)
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
    
    ;; return final result
    (local.get $result)
  )
  
  ;; internal function "sort" - bubble sort implementation
  (func $sort (param $ptr i32) (param $len i32)
    (local $i i32)
    (local $j i32)
    (local $tmp i32)
    (local $addr_i i32)
    (local $addr_j i32)
    (local $val_i i32)
    (local $val_j i32)
    
    ;; outer loop: i from 0 to len-1
    (local.set $i (i32.const 0))
    (block $outer_done
      (loop $outer_loop
        ;; check if outer loop is done
        (br_if $outer_done
          (i32.ge_s
            (local.get $i)
            (i32.sub
              (local.get $len)
              (i32.const 1)
            )
          )
        )
        
        ;; inner loop: j from 0 to len-i-1
        (local.set $j (i32.const 0))
        (block $inner_done
          (loop $inner_loop
            ;; check if inner loop is done
            (br_if $inner_done
              (i32.ge_s
                (local.get $j)
                (i32.sub
                  (i32.sub
                    (local.get $len)
                    (local.get $i)
                  )
                  (i32.const 1)
                )
              )
            )
            
            ;; calculate address
            (local.set $addr_i
              (i32.add
                (local.get $ptr)
                (i32.mul
                  (local.get $j)
                  (i32.const 4)
                )
              )
            )
            (local.set $addr_j
              (i32.add
                (local.get $ptr)
                (i32.mul
                  (i32.add
                    (local.get $j)
                    (i32.const 1)
                  )
                  (i32.const 4)
                )
              )
            )
            
            ;; read value
            (local.set $val_i (i32.load (local.get $addr_i)))
            (local.set $val_j (i32.load (local.get $addr_j)))
            
            ;; if arr[j] > arr[j+1], then swap
            (if
              (i32.gt_s
                (local.get $val_i)
                (local.get $val_j)
              )
              (then
                ;; swap values
                (local.set $tmp (local.get $val_i))
                (i32.store (local.get $addr_i) (local.get $val_j))
                (i32.store (local.get $addr_j) (local.get $tmp))
              )
            )
            
            ;; j++
            (local.set $j
              (i32.add
                (local.get $j)
                (i32.const 1)
              )
            )
            
            ;; continue inner loop
            (br $inner_loop)
          )
        )
        
        ;; i++
        (local.set $i
          (i32.add
            (local.get $i)
            (i32.const 1)
          )
        )
        
        ;; continue outer loop
        (br $outer_loop)
      )
    )
  )
  
  ;; recursive calculation of factorial - large recursive function
  (func $factorial_recursive (export "factorial_recursive") (param $n i32) (result i32)
    (if (result i32)
      (i32.le_s (local.get $n) (i32.const 1))
      (then
        (i32.const 1)  ;; 基准情况：0! = 1! = 1
      )
      (else
        ;; 递归情况：n! = n * (n-1)!
        (i32.mul
          (local.get $n)
          (call $factorial_recursive
            (i32.sub
              (local.get $n)
              (i32.const 1)
            )
          )
        )
      )
    )
  )
  
  ;; export function "process_data" - process array data
  (func $process_data (export "process_data") (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $sum i32)
    (local $product i32)
    (local $current i32)
    
    ;; first sort
    (call $sort
      (local.get $ptr)
      (local.get $len)
    )
    
    ;; initialize accumulated values
    (local.set $sum (i32.const 0))
    (local.set $product (i32.const 1))
    (local.set $i (i32.const 0))
    
    ;; traverse array
    (block $done
      (loop $loop
        ;; check if loop is done
        (br_if $done
          (i32.ge_s
            (local.get $i)
            (local.get $len)
          )
        )
        
        ;; read current value
        (local.set $current
          (i32.load
            (i32.add
              (local.get $ptr)
              (i32.mul
                (local.get $i)
                (i32.const 4)
              )
            )
          )
        )
        
        ;; accumulate
        (local.set $sum
          (i32.add
            (local.get $sum)
            (local.get $current)
          )
        )
        
        ;; multiply
        (local.set $product
          (i32.mul
            (local.get $product)
            (local.get $current)
          )
        )
        
        ;; i++
        (local.set $i
          (i32.add
            (local.get $i)
            (i32.const 1)
          )
        )
        
        ;; continue loop
        (br $loop)
      )
    )
    
    ;; return (sum XOR product)
    (i32.xor
      (local.get $sum)
      (local.get $product)
    )
  )
  
  ;; export function "matrix_multiply" - large matrix multiplication function
  (func $matrix_multiply (export "matrix_multiply") 
      (param $a_ptr i32) (param $b_ptr i32) (param $c_ptr i32) (param $n i32)
    (local $i i32)
    (local $j i32)
    (local $k i32)
    (local $sum i32)
    (local $a_idx i32)
    (local $b_idx i32)
    (local $c_idx i32)
    
    ;; triple loop implementation of matrix multiplication
    (local.set $i (i32.const 0))
    (block $i_done
      (loop $i_loop
        (br_if $i_done
          (i32.ge_s
            (local.get $i)
            (local.get $n)
          )
        )
        
        (local.set $j (i32.const 0))
        (block $j_done
          (loop $j_loop
            (br_if $j_done
              (i32.ge_s
                (local.get $j)
                (local.get $n)
              )
            )
            
            ;; c[i,j] = 0
            (local.set $sum (i32.const 0))
            
            (local.set $k (i32.const 0))
            (block $k_done
              (loop $k_loop
                (br_if $k_done
                  (i32.ge_s
                    (local.get $k)
                    (local.get $n)
                  )
                )
                
                ;; calculate address of a[i,k]
                (local.set $a_idx
                  (i32.add
                    (local.get $a_ptr)
                    (i32.mul
                      (i32.add
                        (i32.mul
                          (local.get $i)
                          (local.get $n)
                        )
                        (local.get $k)
                      )
                      (i32.const 4)
                    )
                  )
                )
                
                ;; calculate address of b[k,j]
                (local.set $b_idx
                  (i32.add
                    (local.get $b_ptr)
                    (i32.mul
                      (i32.add
                        (i32.mul
                          (local.get $k)
                          (local.get $n)
                        )
                        (local.get $j)
                      )
                      (i32.const 4)
                    )
                  )
                )
                
                ;; sum += a[i,k] * b[k,j]
                (local.set $sum
                  (i32.add
                    (local.get $sum)
                    (i32.mul
                      (i32.load (local.get $a_idx))
                      (i32.load (local.get $b_idx))
                    )
                  )
                )
                
                ;; k++
                (local.set $k
                  (i32.add
                    (local.get $k)
                    (i32.const 1)
                  )
                )
                
                (br $k_loop)
              )
            )
            
            ;; calculate address of c[i,j]
            (local.set $c_idx
              (i32.add
                (local.get $c_ptr)
                (i32.mul
                  (i32.add
                    (i32.mul
                      (local.get $i)
                      (local.get $n)
                    )
                    (local.get $j)
                  )
                  (i32.const 4)
                )
              )
            )
            
            ;; store c[i,j] = sum
            (i32.store
              (local.get $c_idx)
              (local.get $sum)
            )
            
            ;; j++
            (local.set $j
              (i32.add
                (local.get $j)
                (i32.const 1)
              )
            )
            
            (br $j_loop)
          )
        )
        
        ;; i++
        (local.set $i
          (i32.add
            (local.get $i)
            (i32.const 1)
          )
        )
        
        (br $i_loop)
      )
    )
  )

  ;; add a large non-exported function with multiple stack balance points
  (func $large_splittable_function
    (param $iterations i32)
    (result i32)
    (local $i i32)
    (local $result i32)
    (local $temp i32)
    
    ;; initialize result
    (local.set $result (i32.const 0))
    (local.set $i (i32.const 0))
    
    ;; this is a safe split point - stack depth is 0
    
    ;; loop calculation
    (block $done
      (loop $loop
        ;; check loop condition
        (br_if $done
          (i32.ge_s
            (local.get $i)
            (local.get $iterations)
          )
        )
        
        ;; this is a safe split point - stack depth is 0
        (local.set $temp (i32.const 10))
        (local.get $temp)
        (local.get $result)
        (i32.add)
        (local.set $result)
        
        ;; this is a safe split point - stack depth is 0
        (local.get $i)
        (i32.const 5)
        (i32.gt_s)
        (if
          (then
            (local.get $result)
            (i32.const 2)
            (i32.mul)
            (local.set $result)
          )
          (else
            (local.get $result)
            (i32.const 5)
            (i32.add)
            (local.set $result)
          )
        )
        
        ;; this is a safe split point - stack depth is 0
        (local.get $result)
        (i32.const 3)
        (i32.sub)
        (local.set $result)
        
        ;; this is a safe split point - stack depth is 0
        (local.get $result)
        (local.get $i)
        (i32.const 2)
        (i32.mul)
        (i32.add)
        (local.set $result)
        
        ;; this is a safe split point - stack depth is 0
        (local.get $i)
        (i32.const 1)
        (i32.add)
        (local.set $i)
        
        ;; this is a safe split point - stack depth is 0
        (local.get $i)
        (i32.const 3)
        (i32.rem_s)
        (i32.eqz)
        (if
          (then
            (local.get $result)
            (i32.const 100)
            (i32.add)
            (local.set $result)
          )
        )
        
        ;; this is a safe split point - stack depth is 0
        
        ;; add some extra instructions to ensure the function is large enough
        (local.get $i)
        (i32.const 10)
        (i32.rem_s)
        (i32.const 0)
        (i32.eq)
        (if
          (then
            (local.get $result)
            (i32.const 50)
            (i32.add)
            (local.set $result)
          )
        )
        
        ;; this is a safe split point - stack depth is 0
        (local.get $result)
        (i32.const 7)
        (i32.rem_s)
        (i32.const 0)
        (i32.eq)
        (if
          (then
            (local.get $result)
            (i32.const 25)
            (i32.add)
            (local.set $result)
          )
        )
        
        ;; this is a safe split point - stack depth is 0
        (br $loop)
      )
    )
    
    ;; return final result
    (local.get $result)
  )

  ;; memory definition
  (memory (export "memory") 1)
) 