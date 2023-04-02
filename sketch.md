# RTL - Register Transfer Language
| RTL                                     | Description                                                                                                                      |
|-----------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| `cmp %0, %1, %2`                        | Compare - Compare %0 with %1 and place the flag in %2                                                                            |
| `jmpne %0, %1`                          | Jump Not Equal - Jump to the address %1 if the flag %0 is NE (Not-Equal).                                                        |
| `jmp %0`                                | Jump - Unconditionally jump to the address %0                                                                                    |
| `mv %0, %1`                             | Move - Move the value in %1 into %0                                                                                              |
| `usub %0, %1, %2`                       | Unsigned subtraction - Subtracts %0 with %1 and place the unsigned result in %2                                                  |
| `umul %0, %1, %2`                       | Unsigned multiply - Multiply %0 with %1 and place the unsigned result in %2                                                      |
| `callfx %0, <reg list 0>, <reg list 1>` | Call fast special - Calls %0 with the arguments bound to registers in reg list 0 and return values bound to values in reg list 1 |
| `phi %0, <reg list 0>`                  | https://en.wikipedia.org/wiki/Static_single-assignment_form#Converting_to_SSA                                                    |
| `ret`                                   | Returns from the current procedure                                                                                               |

# AIR - Allocation Intermediate Representation
| AIR                     | Constraints | RTL Semi-equivalent   | AMD64         |
|-------------------------|-------------|-----------------------|---------------|
| `amd64_cmp %0, %1, %2`  | `%2 = %ZF`  | `cmp %0, %1, %2`      | `cmp %0 %1`   |
| `amd64_jne %0, %1`      | `%0 = %ZF`  | `jmpne %0, %1`        | `jne %1`      |
| `amd64_jmp %0`          |             | `jmp %0`              | `jmp %0`      |
| `amd64_mov %0, %1`      |             | `mv %0, %1`           | `mov %0, %1`  |
| `amd64_sub %0, %1, %2`  | `%0 = %2`   | `sub %0, %1, %2`      | `sub %0, %1`  |
| `amd64_imul %0, %1, %2` | `%0 = %2`   | `umul %0, %1, %2`     | `imul %0, %1` |
| `amd64_call %0`         |             | `callfx %0, N/A, N/A` | `call %0`     |
| `amd64_push %0`         |             | N/A                   | `push %0`     |
| `amd64_pop %0`          |             | N/A                   | `pop %0`      |
| `amd64_ret`             |             | `ret`                 | `ret`         |

## Factorial function
```rtl
fn fac(%0: u32): (%5: u32) => fac_0
fac_0 =>
  cmp %0, 1, %1
  jmpne %1, @fac_1
  mv %2, 1
  jmp fac_2
  
fac_1 =>
  usub %0, 1, %2
  callfx @fac [%2] [%3]
  umul %0, %3, %4
  jmp @fac_2

fac_2 =>
  phi %5, [%4, %2] 
  ret
```
### Instruction selection and scheduling pass
```rtl
fn fac(%0: u32): (%5: u32) => fac_0
fac_0 =>
  cmp %0, 1, %1 < amd64_cmp %0, 1, %1 >
  jmpne %1, @fac_1 < amd64_jne %1, @fac_1 >
  mv %2, 1 < amd64_mov %2, 1 >
  jmp @fac_2_phi1 < amd64_jmp @fac_2_phi1 >
  
fac_1 =>
  usub %0, 1, %2 < amd64_sub %0, 1, %2 >
  callfx @fac [%2] [%3] <
    GA_mvarg @fac, 0, %2
    amd64_call @fac
    GA_mvret @fac, 0, %3
  >
  umul %3, %0, %4 < amd64_imul %3, %0, %4 >
  jmp @fac_2_phi0 < amd64_jmp @fac_2_phi0 >

fac_2_phi0 =>
  mv %5, %4 < amd64_mov %5, %4 >
  ret < amd64_ret >

fac_2_phi1 =>
  mv %5, %2 < amd64_mov %5, %2 >
  ret < amd64_ret >
```

### Register allocation pass
After register allocation:
```mir
fn fac(%ecx: u32): (%eax: u32) => fac_0

fac_0 =>
  amd64_cmp %ecx, 1, %zf
  amd64_jne %zf, @fac_1
  amd64_mov %eax, 1
  amd64_jmp @fac_2_phi1

fac_1 =>
  # ECX gets moved to the stack because of recursive call
  amd64_push %ecx
  amd64_sub %ecx, 1, %ecx
  amd64_call @fac
  # mov %eax, %eax (optimized out)
  # pop ECX off the stack
  amd64_pop %ecx
  amd64_imul { %eax, %ecx, %eax }
  amd64_jmp @fac_2_phi0

fac_2_phi0 =>
  # amd64_mov eax, eax (optimized out)
  amd64_ret

fac_2_phi1 =>
  # amd64_mov eax, eax (optimized out)
  amd64_ret
```

### GCSE (Global Common Subexpression Elimination)
```mir
fn fac(%ecx: u32): (%eax: u32) => fac_0

fac_0 =>
  amd64_cmp %ecx, 1, %zf
  amd64_jne %zf, @fac_1
  amd64_mov %eax, 1
  amd64_ret

fac_1 =>
  # ECX gets moved to the stack
  amd64_push %ecx
  amd64_sub %ecx, 1, %ecx
  amd64_call @fac
  # mov %eax, %eax (optimized out)
  # pop ECX off the stack
  amd64_pop %ecx
  amd64_imul %ecx, %eax, %eax
  amd64_ret
```

### Output
Final code:
```asm
fac_0:
  cmp ecx, 1
  jne fac_1
  mov eax, 1
  ret

fac_1:
  push ecx
  sub ecx, 1
  call fac_0
  pop ecx
  imul eax, ecx
  ret
```
