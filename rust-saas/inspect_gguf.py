import struct
path = r'J:\gemma4\models--unsloth--gemma-4-E4B-it-GGUF\snapshots\ce152932ac27bc40bc9c727386760424d50bb456\gemma-4-E4B-it-Q4_0.gguf'
with open(path, 'rb') as f:
    magic = f.read(4)
    print('magic', magic)
    version = struct.unpack('<I', f.read(4))[0]
    n_tensors = struct.unpack('<Q', f.read(8))[0]
    n_kv = struct.unpack('<Q', f.read(8))[0]
    print('version', version, 'n_tensors', n_tensors, 'n_kv', n_kv)

    def read_str():
        l = struct.unpack('<Q', f.read(8))[0]
        return f.read(l).decode('utf-8', errors='replace')

    def read_val():
        vt = struct.unpack('<I', f.read(4))[0]
        if vt == 0: return ('u8', struct.unpack('<B', f.read(1))[0])
        if vt == 1: return ('i8', struct.unpack('<b', f.read(1))[0])
        if vt == 2: return ('u16', struct.unpack('<H', f.read(2))[0])
        if vt == 3: return ('i16', struct.unpack('<h', f.read(2))[0])
        if vt == 4: return ('u32', struct.unpack('<I', f.read(4))[0])
        if vt == 5: return ('i32', struct.unpack('<i', f.read(4))[0])
        if vt == 6: return ('f32', struct.unpack('<f', f.read(4))[0])
        if vt == 7: return ('bool', struct.unpack('<B', f.read(1))[0] != 0)
        if vt == 8:
            l = struct.unpack('<Q', f.read(8))[0]
            return ('str', f.read(l).decode('utf-8', errors='replace'))
        if vt == 9:
            et = struct.unpack('<I', f.read(4))[0]
            n = struct.unpack('<Q', f.read(8))[0]
            arr = []
            for _ in range(n):
                arr.append(read_val()[1])
            return ('array', et, n, arr[:5])
        if vt == 10: return ('u64', struct.unpack('<Q', f.read(8))[0])
        if vt == 11: return ('i64', struct.unpack('<q', f.read(8))[0])
        if vt == 12: return ('f64', struct.unpack('<d', f.read(8))[0])
        return ('unknown', vt)

    for i in range(n_kv):
        key = read_str()
        val = read_val()
        if val[0] == 'array' and val[1] == 7:
            print('BOOL ARRAY:', key, 'len', val[2])
        elif val[0] == 'array':
            print('ARRAY type', val[1], key, 'len', val[2])
