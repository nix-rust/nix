# PYTHONPATH=/home/cmr/llvm/tools/clang/bindings/python LD_LIBRARY_PATH=/usr/lib ./find_ioctls.py

import os
import clang.cindex as c

header_paths = ["/usr/include"]

idx = c.Index.create()
args = ['-E', '-x', 'c', '-I/usr/lib/clang/3.6.0/include/']
options = c.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD | c.TranslationUnit.PARSE_SKIP_FUNCTION_BODIES | c.TranslationUnit.PARSE_INCOMPLETE

ioctls = []

for p in header_paths:
    for (dirp, dirn, fnames) in os.walk(p):
        for f in fnames:
            if f.endswith('.h'): # try to avoid C++ headers.
                tu = idx.parse(os.path.join(dirp, f), args=args,
                        options=options)
                failed = False
                for diag in tu.diagnostics:
                    if diag.severity > c.Diagnostic.Warning:
                        failed = True
                        break
                if failed:
                    continue
                for cx in tu.cursor.walk_preorder():
                    if cx.kind == c.CursorKind.MACRO_DEFINITION:
                        if "IOC" in cx.spelling and cx.spelling.isupper():
                            ioctls.append(list(tok.spelling for tok in cx.get_tokens()))

for ioctl in ioctls:
    print(ioctl)
