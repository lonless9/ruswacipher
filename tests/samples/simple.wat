(module
  ;; export function "add" 
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    (i32.add 
      (local.get $a) 
      (local.get $b)
    )
  )
  
  ;; export function "sub"
  (func $sub (export "sub") (param $a i32) (param $b i32) (result i32)
    (i32.sub
      (local.get $a)
      (local.get $b)
    )
  )
  
  ;; export function "mul"
  (func $mul (export "mul") (param $a i32) (param $b i32) (result i32)
    (i32.mul
      (local.get $a)
      (local.get $b)
    )
  )
  
  ;; internal function "factorial" - not exported
  (func $factorial (param $n i32) (result i32)
    (local $result i32)
    (local $i i32)
    
    ;; initialize result to 1
    (local.set $result (i32.const 1))
    ;; initialize counter
    (local.set $i (i32.const 2))
    
    ;; loop until i > n
    (block $done
      (loop $loop
        ;; if i > n, break the loop
        (br_if $done
          (i32.gt_s
            (local.get $i)
            (local.get $n)
          )
        )
        
        ;; result = result * i
        (local.set $result
          (i32.mul
            (local.get $result)
            (local.get $i)
          )
        )
        
        ;; i = i + 1
        (local.set $i
          (i32.add
            (local.get $i)
            (i32.const 1)
          )
        )
        
        ;; return to loop start
        (br $loop)
      )
    )
    
    ;; return factorial value
    (local.get $result)
  )
  
  ;; export function "fac" - calculate factorial and return
  (func $fac (export "fac") (param $n i32) (result i32)
    (call $factorial (local.get $n))
  )
  
  ;; export function "fibonacci" - calculate fibonacci and return
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (local $i i32)
    (local $prev i32)
    (local $curr i32)
    (local $next i32)
    
    ;; boundary check
    (if (result i32)
      (i32.le_s (local.get $n) (i32.const 1))
      (then 
        (local.get $n)  ;; if n <= 1, return n directly
      )
      (else
        ;; initialize prev=0, curr=1, i=2
        (local.set $prev (i32.const 0))
        (local.set $curr (i32.const 1))
        (local.set $i (i32.const 2))
        
        ;; loop to calculate fibonacci sequence
        (block $done
          (loop $loop
            ;; if i > n, end the loop
            (br_if $done
              (i32.gt_s
                (local.get $i)
                (local.get $n)
              )
            )
            
            ;; next = prev + curr
            (local.set $next
              (i32.add
                (local.get $prev)
                (local.get $curr)
              )
            )
            
            ;; prev = curr
            (local.set $prev (local.get $curr))
            
            ;; curr = next
            (local.set $curr (local.get $next))
            
            ;; i = i + 1
            (local.set $i
              (i32.add
                (local.get $i)
                (i32.const 1)
              )
            )
            
            ;; return to loop start
            (br $loop)
          )
        )
        
        ;; return final fibonacci number
        (local.get $curr)
      )
    )
  )
) 