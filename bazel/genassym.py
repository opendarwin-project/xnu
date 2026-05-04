import sys
import subprocess
import re

# Usage: genassym.py <cc_command...> -- <input_c> <output_s>

def main():
    args = sys.argv[1:]
    try:
        separator_index = args.index('--')
    except ValueError:
        print("Missing '--' separator")
        sys.exit(1)
    
    cc_cmd = args[:separator_index]
    files = args[separator_index+1:]
    
    if len(files) != 2:
        print("Usage: genassym.py <cc_command...> -- <input_c> <output_s>")
        sys.exit(1)
    
    input_c = files[0]
    output_s = files[1]
    
    # We use -o - to output to stdout
    cmd = cc_cmd + [input_c, '-o', '-']
    process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    stdout, stderr = process.communicate()
    
    if process.returncode != 0:
        print(f"CC failed with return code {process.returncode}")
        print(stderr)
        sys.exit(process.returncode)
    
    lines = stdout.splitlines()
    output_lines = []
    
    i = 0
    while i < len(lines):
        line = lines[i]
        if 'DEFINITION__define__' in line:
            # Join with next line
            next_line = ""
            if i + 1 < len(lines):
                next_line = lines[i+1].strip()
            
            combined = line.strip() + next_line
            # Match DEFINITION__define__NAME: .ascii "VAL"
            # Some compilers might put : on the same line or next line.
            match = re.search(r'DEFINITION__define__([^:]*):.*ascii.*"[\$#]*([-0-9#]*)".*$', combined)
            if match:
                name = match.group(1)
                val = match.group(2)
                # Clean value (remove $ or #)
                clean_val = val.lstrip('$#')
                output_lines.append(f"#define {name} {clean_val}")
                output_lines.append(f"#define {name}_NUM {clean_val}")
            i += 1
        i += 1

    with open(output_s, 'w') as f:
        f.write('\n'.join(output_lines) + '\n')

if __name__ == '__main__':
    main()
