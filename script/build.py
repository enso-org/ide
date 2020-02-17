#!/usr/bin/env python3

from distutils.dir_util import copy_tree
import os
import re
import shutil
import subprocess

def read_file(path):
    with open(path) as f:
        return f.read()


def write_file(path, contents):
    with open(path, 'w') as f:
        f.write(contents)


def patch_file(path, patcher):
    print(f'Patching {path}')
    code_to_patch = read_file(path)
    patched_code = patcher(code_to_patch)
    write_file(path, patched_code)


# Workaround fix by wdanilo, see: https://github.com/rustwasm/wasm-pack/issues/790
def js_workaround_patcher(code):
    code = re.sub(r'(?ms)if \(\(typeof URL.*}\);', 'return imports', code)
    code = re.sub(r'(?ms)if \(typeof module.*let result', 'let result', code)
    code = f'{code}\nexport function after_load(w,m) {{ wasm = w; init.__wbindgen_wasm_module = m;}}'
    return code

repo_root = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))


print(f'Build Working Directory: {repo_root}')
os.chdir(repo_root)

print('Building with wasm-pack...')
subprocess.check_call(['wasm-pack', 'build', '--target', 'web', '--no-typescript', '--out-dir', '../../target/web', 'lib/gui'], shell=True)
patch_file('target/web/gui.js', js_workaround_patcher)
shutil.move('target/web/gui_bg.wasm', 'target/web/gui.wasm')

# TODO [mwu] It should be possible to drop gzip program dependency by using Python's gzip library.
subprocess.check_call(['gzip', '--keep', '--best', '--force', 'target/web/gui.wasm'], shell=True)

# Note [MWU] We build to provisional location and patch files there before copying, so the backpack don't get errors
#            from processing unpatched files.
#            Also, here we copy into (overwriting), without removing old files. Backpack on Windows does not tolerate
#            removing files it watches.
copy_tree('target/web', 'app/src-rust-gen')

